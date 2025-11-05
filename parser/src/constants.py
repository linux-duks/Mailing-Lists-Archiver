import polars as pl

PARQUET_COLS_SCHEMA = {
    "from": pl.String,
    "to": pl.List(pl.String),
    "cc": pl.List(pl.String),
    "subject": pl.String,
    "date": pl.Datetime,
    "message-id": pl.String,
    "in-reply-to": pl.String,
    "references": pl.List(pl.String),
    "x-mailing-list": pl.String,
    "body": pl.String,
    "trailers": pl.List(
        pl.Struct({"attribution": pl.String, "identification": pl.String})
    ),
    "code": pl.String,
    "raw_body": pl.String,
}

BEFORE_SIGNED = "body"
AFTER_SIGNED = "code"
SIGNED_BLOCK = "trailers"

SINGLE_VALUED_COLS = [
    "from",
    "to",
    "subject",
    "date",
    "message-id",
    "in-reply-to",
    "x-mailing-list",
    "body",
    "code",
]

N_PROC = 4

REDO_FAILED_PARSES = (
    False  # Parse only the emails that were unsuccessfully parsed on previous runs.
)

LISTS_TO_PARSE = []

KEYS_MASK = [
    "from",
    "to",
    "cc",
    "subject",
    "date",
    "message-id",
    "in-reply-to",
    "references",
    "x-mailing-list",
    BEFORE_SIGNED,
    SIGNED_BLOCK,
    AFTER_SIGNED,
]
