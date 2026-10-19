// Mirrors hyperswitch-control-center/src/utils/Typography.res + TypographyUtils.res
// (body / heading / code scales as Tailwind class strings).
type weights = {semibold: string, medium: string, regular: string}
type sizes = {lg: weights, md: weights, sm: weights, xs: weights}

let body: sizes = {
  lg: {
    semibold: "text-fs-16 leading-24 font-semibold",
    medium: "text-fs-16 leading-24 font-medium",
    regular: "text-fs-16 leading-24 font-normal",
  },
  md: {
    semibold: "text-fs-14 leading-20 font-semibold",
    medium: "text-fs-14 leading-20 font-medium",
    regular: "text-fs-14 leading-20 font-normal",
  },
  sm: {
    semibold: "text-fs-12 leading-18 font-semibold",
    medium: "text-fs-12 leading-18 font-medium",
    regular: "text-fs-12 leading-18 font-normal",
  },
  xs: {
    semibold: "text-fs-10 leading-14 font-semibold",
    medium: "text-fs-10 leading-14 font-medium",
    regular: "text-fs-10 leading-14 font-normal",
  },
}

let heading: sizes = {
  lg: {
    semibold: "text-fs-24 leading-32 font-semibold",
    medium: "text-fs-24 leading-32 font-medium",
    regular: "text-fs-24 leading-32 font-normal",
  },
  md: {
    semibold: "text-fs-20 leading-26 font-semibold",
    medium: "text-fs-20 leading-26 font-medium",
    regular: "text-fs-20 leading-26 font-normal",
  },
  sm: {
    semibold: "text-fs-18 leading-24 font-semibold",
    medium: "text-fs-18 leading-24 font-medium",
    regular: "text-fs-18 leading-24 font-normal",
  },
  xs: {
    semibold: "text-fs-16 leading-24 font-semibold",
    medium: "text-fs-16 leading-24 font-medium",
    regular: "text-fs-16 leading-24 font-normal",
  },
}

let code: sizes = {
  lg: {
    semibold: "text-fs-14 leading-18 font-semibold font-jetbrains-mono",
    medium: "text-fs-14 leading-18 font-medium font-jetbrains-mono",
    regular: "text-fs-14 leading-18 font-normal font-jetbrains-mono",
  },
  md: {
    semibold: "text-fs-12 leading-18 font-semibold font-jetbrains-mono",
    medium: "text-fs-12 leading-18 font-medium font-jetbrains-mono",
    regular: "text-fs-12 leading-18 font-normal font-jetbrains-mono",
  },
  sm: {
    semibold: "text-fs-10 leading-14 font-semibold font-jetbrains-mono",
    medium: "text-fs-10 leading-14 font-medium font-jetbrains-mono",
    regular: "text-fs-10 leading-14 font-normal font-jetbrains-mono",
  },
  xs: {
    semibold: "text-fs-10 leading-14 font-semibold font-jetbrains-mono",
    medium: "text-fs-10 leading-14 font-medium font-jetbrains-mono",
    regular: "text-fs-10 leading-14 font-normal font-jetbrains-mono",
  },
}
