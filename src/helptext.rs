//! The parser's own texts, byte for byte as the
//! Python's argparse prints them at the default
//! width: the two help screens and the two usage
//! lines that head its errors.

pub const TOP_USAGE: &str = "usage: secondtake [-h] {compare} ...";

pub const TOP_HELP: &str = "usage: secondtake [-h] {compare} ...

positional arguments:
  {compare}
    compare   the matched comparison

options:
  -h, --help  show this help message and exit
";

pub const COMPARE_USAGE: &str = "usage: secondtake compare [-h] --registration REGISTRATION [--judge JUDGE]
                          [-o OUTPUT]
                          before after";

pub const COMPARE_HELP: &str = "usage: secondtake compare [-h] --registration REGISTRATION [--judge JUDGE]
                          [-o OUTPUT]
                          before after

positional arguments:
  before
  after

options:
  -h, --help            show this help message and exit
  --registration REGISTRATION
  --judge JUDGE         a judgments file to replay, or live
  -o OUTPUT, --output OUTPUT
";
