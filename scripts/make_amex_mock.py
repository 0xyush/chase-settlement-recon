#!/usr/bin/env python3
"""Make a mock American Express GRRCN file (v4.01 layout) for the Amex rows in a Chase DFR file.

Chase conveys Amex: its DFR lists the Amex sales but pays none of them. Amex pays the merchant
directly and reports it in its own GRRCN file. This script writes that file for the Amex rows of
one DFR day, so the two can be matched: Chase ACT0010 row -> GRRCN TRANSACTN -> GRRCN SUMMARY
(the Amex deposit).

Everything is made up: merchant numbers, payment number, transaction IDs and the discount rate.
Field order follows the April 2024 GRRCN v4.01 layout, as read by
https://github.com/hotter6163/amex-grrcn-viewer (src/lib/grrcn-parser.ts). Not checked against a
real file yet.

Usage: scripts/make_amex_mock.py engines/orbital/samples/0000999999.261005.d.B105.dfr samples/amex
"""
import csv
import io
import sys
from datetime import date, timedelta
from pathlib import Path

SE_NUMBER = "9999999999"          # Amex Service Establishment number (merchant ID), made up
PAYMENT_NUMBER = "9261008001"     # Amex payment number: one per deposit to the bank
DISCOUNT_BP = 250                 # 2.50% Amex discount, made up
FEE_CODE = "1A"                   # as in Amex's public v2.01 sample
MCC = "4784"                      # tolls and bridge fees
PAY_DELAY_DAYS = 3                # Amex pays a few days after submission


def amount16(cents: int) -> str:
    """Amex 16-byte amount: sign (space or '-') + 15 digits, 2 implied decimals for USD."""
    return f"{'-' if cents < 0 else ' '}{abs(cents):015d}"


def amount22(micro: int) -> str:
    """Amex 22-byte pricing amount: sign + 15 whole digits + 6 decimals."""
    return f"{'-' if micro < 0 else ' '}{abs(micro):021d}"


def rate7(bp: int) -> str:
    """Amex 7-byte rate: sign + 1 whole digit + 5 decimals. 250 bp -> ' 002500'."""
    return f" {bp * 10:06d}"


def cents_of(text: str) -> int:
    text = text.strip()
    negative = text.startswith("-")
    whole, _, frac = text.lstrip("-").partition(".")
    cents = int(whole or "0") * 100 + int((frac + "00")[:2])
    return -cents if negative else cents


def yyyymmdd(mdy: str) -> str:
    month, day, year = mdy.split("/")
    return f"{year}{month}{day}"


def record(kind: str, last: int, values: dict[int, str]) -> list[str]:
    """One GRRCN record: field 0 is the record type, then fields 1..last by position."""
    assert max(values) <= last, (kind, max(values))
    return [kind] + [values.get(i, "") for i in range(1, last + 1)]


def amex_rows(dfr_text: str) -> tuple[str, list[dict]]:
    """The Amex (MOP AX) rows of the ACT0010 section, and the file's activity date."""
    rows, activity = [], ""
    for line in dfr_text.splitlines():
        if line.startswith("HACT0010"):
            activity = line.split("\t")[2]
        if not line.startswith("RACT0010"):
            continue
        f = next(csv.reader([line], delimiter="\t"))
        if f[14] != "AX":
            continue
        rows.append({
            "order": f[9], "account": f[11], "cents": cents_of(f[13]), "action": f[15],
            "auth_date": f[16], "auth_code": f[17], "submitted": f[1],
        })
    return activity, rows


def build(dfr_text: str) -> tuple[str, str, dict]:
    activity, rows = amex_rows(dfr_text)
    sub_date = yyyymmdd(activity)
    month, day, year = (int(x) for x in activity.split("/"))
    pay = date(year, month, day) + timedelta(days=PAY_DELAY_DAYS)
    pay_date = pay.strftime("%Y%m%d")
    created = pay

    # Fields 1 to 5 open every SUMMARY, SUBMISSION, TRANSACTN and TXNPRICING record.
    head = {1: SE_NUMBER, 2: "002", 3: PAYMENT_NUMBER, 4: pay_date, 5: "USD"}
    txns = []
    gross = discount = debit_gross = credit_gross = 0
    for i, r in enumerate(rows):
        cents = r["cents"]
        disc = (cents * DISCOUNT_BP + (5000 if cents >= 0 else -5000)) // 10_000
        gross += cents
        discount += disc
        if cents >= 0:
            debit_gross += cents
        else:
            credit_gross += cents
        txn_date = yyyymmdd(r["auth_date"] or r["submitted"])
        txn_id = f"{int(sub_date) % 1_000_000:06d}{i + 1:09d}"
        txn = record("TRANSACTN", 45, {
            **head, 6: SE_NUMBER, 7: sub_date, 8: sub_date, 9: "000000000000001", 10: "USD",
            12: r["order"],  # Invoice Reference Number: assumed to carry Chase's order number
            14: r["account"], 16: amount16(cents), 17: amount16(cents), 18: txn_date,
            20: txn_id, 21: r["auth_code"], 23: MCC,
        })
        pricing = record("TXNPRICING", 26, {
            **head, 6: SE_NUMBER, 9: r["order"], 11: r["account"], 12: amount16(cents), 13: txn_date,
            14: FEE_CODE, 16: amount22(0), 17: rate7(DISCOUNT_BP), 18: amount22(disc * 10_000),
            19: amount16(0), 20: amount16(disc), 21: amount22(0), 22: amount22(disc * 10_000),
            23: amount16(cents),
        })
        txns.append((txn, pricing))

    net = gross - discount
    summary = record("SUMMARY", 18, {
        **head, 7: amount16(net), 8: amount16(gross), 9: amount16(discount), 10: amount16(0),
        11: amount16(0), 12: amount16(0), 13: amount16(0), 15: "XXXXXX1234",
    })
    submission = record("SUBMISSION", 33, {
        **head, 6: SE_NUMBER, 7: sub_date, 8: sub_date, 9: "000000000000001", 10: "USD",
        13: amount16(gross), 14: amount16(gross), 15: amount16(discount), 16: amount16(0),
        17: amount16(0), 18: amount16(net), 19: rate7(DISCOUNT_BP), 21: f"{len(rows):06d}",
        30: amount16(debit_gross), 31: amount16(credit_gross),
    })

    records = [["HEADER", created.strftime("%Y%m%d"), "021500", "0000000001", "GRRCN", "SAMPLEMERCHANT", "4.01"],
               summary, submission]
    for txn, pricing in txns:
        records += [txn, pricing]
    records.append(["TRAILER", "0000000001", f"{len(records) + 1:010d}"])

    out = io.StringIO()
    csv.writer(out, quoting=csv.QUOTE_ALL, lineterminator="\r\n").writerows(records)
    name = f"SAMPLEMERCHANT.GRRCN.{pay.strftime('%y%m%d')}021500.MOCK.csv"
    totals = {"rows": len(rows), "gross": gross, "discount": discount, "net": net, "payment_date": pay_date}
    return name, out.getvalue(), totals


def main() -> None:
    dfr, out_dir = Path(sys.argv[1]), Path(sys.argv[2])
    name, text, totals = build(dfr.read_text())
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / name).write_text(text)
    print(f"wrote {out_dir / name}: {totals}")


if __name__ == "__main__":
    main()
