// Transformation configs for each Chase report.
// Same idea as the real POST /transformations/v2/configs request, in a flatter shape.

export const PRESETS = {
  "deposit_details": {
    "report": "Deposit Details",
    "parsing_config": {
      "file_format": "csv",
      "header_row_index": 2,
      "footer_marker": "END"
    },
    "skip_configs": [],
    "fields": {
      "currency": {
        "identifier": "Presentment Currency Code",
        "transformation_rules": [
          {
            "type": "trim"
          }
        ]
      },
      "amount": {
        "identifier": "Transaction Amount in Presentment Currency",
        "unit": "major_unit",
        "transformation_rules": [
          {
            "type": "trim"
          },
          {
            "type": "absolute"
          }
        ]
      },
      "balance_direction": {
        "identifier": "Merchant Action Code",
        "credit_values": [
          "DP",
          "DO"
        ],
        "debit_values": [
          "RF"
        ],
        "transformation_rules": [
          {
            "type": "trim"
          }
        ]
      },
      "effective_at": {
        "identifier": "Submission Date",
        "date_format": {
          "order": "month_day_year",
          "delimiter": "slash"
        }
      },
      "order_id": {
        "identifier": "Merchant Order Number"
      }
    },
    "metadata_fields": [
      {
        "identifier": "Payment Method Code",
        "field_name": "card_brand",
        "required": true,
        "exposed": true
      },
      {
        "identifier": "Account Number",
        "field_name": "card_last4",
        "required": false,
        "transformation_rules": [
          {
            "type": "regex",
            "pattern": "(\\d{4})?$",
            "group": 1
          }
        ]
      },
      {
        "identifier": "Transaction Division External Identifier",
        "field_name": "division_id",
        "required": true,
        "exposed": true,
        "searchable": true
      },
      {
        "identifier": "Transaction Type Code",
        "field_name": "txn_type",
        "required": false
      },
      {
        "identifier": "Total Interchange Amount",
        "field_name": "interchange_fee",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Total Assessment Amount",
        "field_name": "assessment_fee",
        "type": "major_unit",
        "required": false
      }
    ],
    "unique_constraint": [
      "order_id",
      "balance_direction"
    ]
  },
  "transaction_fee_details": {
    "report": "Transaction Fee Details",
    "parsing_config": {
      "file_format": "csv",
      "header_row_index": 2,
      "footer_marker": "END"
    },
    "skip_configs": [
      {
        "conditions": [
          {
            "identifier": "Merchant Order Number",
            "operator": "equals",
            "value": null
          }
        ]
      }
    ],
    "fields": {
      "currency": {
        "identifier": "Presentment Currency Code",
        "transformation_rules": [
          {
            "type": "trim"
          }
        ]
      },
      "amount": {
        "identifier": "Transaction Amount in Presentment Currency",
        "unit": "major_unit",
        "transformation_rules": [
          {
            "type": "trim"
          },
          {
            "type": "absolute"
          }
        ]
      },
      "balance_direction": {
        "identifier": "Transaction Amount in Presentment Currency",
        "credit_values": [
          "credit"
        ],
        "debit_values": [
          "debit"
        ],
        "transformation_rules": [
          {
            "type": "trim"
          },
          {
            "type": "starts_with",
            "prefix": "-",
            "then": "debit",
            "otherwise": "credit"
          }
        ]
      },
      "effective_at": {
        "identifier": "Activity Date",
        "date_format": {
          "order": "month_day_year",
          "delimiter": "slash"
        }
      },
      "order_id": {
        "identifier": "Merchant Order Number"
      }
    },
    "metadata_fields": [
      {
        "identifier": "Payment Method Code",
        "field_name": "card_brand",
        "required": true,
        "exposed": true
      },
      {
        "identifier": "Entity Identifier",
        "field_name": "entity_id",
        "required": true,
        "exposed": true,
        "searchable": true
      },
      {
        "identifier": "Settled Deposit Amount",
        "field_name": "settled_amount",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Interchange & Assessment Fees",
        "field_name": "interchange_fees",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Processing Fees",
        "field_name": "processing_fees",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Net Financial Activity",
        "field_name": "net_amount",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Fund Transfer Instruction Number",
        "field_name": "fund_transfer_instruction",
        "required": false
      }
    ],
    "unique_constraint": [
      "order_id",
      "balance_direction"
    ]
  },
  "funded_txn_fee_details": {
    "report": "Funded Transaction Fee Details",
    "parsing_config": {
      "file_format": "csv",
      "header_row_index": 2,
      "footer_marker": "END"
    },
    "skip_configs": [
      {
        "conditions": [
          {
            "identifier": "Action Type Code",
            "operator": "equals",
            "value": "AUTH"
          }
        ]
      },
      {
        "conditions": [
          {
            "identifier": "Action Type Code",
            "operator": "equals",
            "value": "OTHER"
          }
        ]
      }
    ],
    "fields": {
      "currency": {
        "identifier": "Presentment Currency Code",
        "transformation_rules": [
          {
            "type": "trim"
          }
        ]
      },
      "amount": {
        "identifier": "Presentment Amount",
        "unit": "major_unit",
        "transformation_rules": [
          {
            "type": "trim"
          },
          {
            "type": "absolute"
          }
        ]
      },
      "balance_direction": {
        "identifier": "Sale/Refund Code",
        "credit_values": [
          "S"
        ],
        "debit_values": [
          "R"
        ],
        "transformation_rules": [
          {
            "type": "trim"
          },
          {
            "type": "default_value",
            "value": "R"
          }
        ]
      },
      "effective_at": {
        "identifier": "Activity Date",
        "date_format": {
          "order": "month_day_year",
          "delimiter": "slash"
        }
      },
      "order_id": {
        "identifier": "Merchant Order Number"
      }
    },
    "metadata_fields": [
      {
        "identifier": "Payment Method Code",
        "field_name": "card_brand",
        "required": false,
        "exposed": true
      },
      {
        "identifier": "Action Type Code",
        "field_name": "action_type",
        "required": true,
        "exposed": true
      },
      {
        "identifier": "Acquirer Reference Number",
        "field_name": "arn",
        "required": false,
        "exposed": true,
        "searchable": true
      },
      {
        "identifier": "Fund Transfer Identifier",
        "field_name": "deposit_id",
        "required": true,
        "exposed": true,
        "searchable": true
      },
      {
        "identifier": "Fund Transfer Effective Date",
        "field_name": "deposit_date",
        "required": false
      },
      {
        "identifier": "Net Financial Activity",
        "field_name": "net_amount",
        "type": "major_unit",
        "required": false
      },
      {
        "identifier": "Processing Fees",
        "field_name": "processing_fees",
        "type": "major_unit",
        "required": false
      }
    ],
    "unique_constraint": [
      "order_id",
      "balance_direction"
    ]
  }
};

export const SAMPLES = [
  {
    "file": "Deposit_Details_Preset_Report.csv",
    "label": "Deposit Details"
  },
  {
    "file": "Transaction_Fee_Details_Preset_Report.csv",
    "label": "Transaction Fee Details"
  },
  {
    "file": "Funded_Transaction_Fee_Details_Preset_Report.csv",
    "label": "Funded Txn Fee Details"
  }
];
