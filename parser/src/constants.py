import polars as pl

PARQUET_COLS_SCHEMA = {'from' : pl.String,
                 'to': pl.String,
                 'cc': pl.List(pl.String),
                 'subject' : pl.String,
                 'date' : pl.Datetime,
                 'message-id': pl.String,
                 'in-reply-to': pl.String,
                 'references': pl.List(pl.String),
                 'x-mailing-list' : pl.String,
                 'body' : pl.String,
                 'trailers': pl.List(pl.String),
                 'code': pl.String}

BEFORE_SIGNED = "body"
AFTER_SIGNED = "code"
SIGNED_BLOCK = "trailers"

N_PROC = 4

FORCE_REPARSE = True #Always reparse every email on list. Otherwise parse only new emails.

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
  AFTER_SIGNED
]