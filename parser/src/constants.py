import polars as pl

PARQUET_COLS_SCHEMA = {'From' : pl.String,
                 'To': pl.String,
                 'Cc': pl.List(pl.String),
                 'Subject' : pl.String,
                 'Date' : pl.Datetime,
                 'Message-ID': pl.String,
                 'In-Reply-To': pl.String,
                 'References': pl.List(pl.String),
                 'X-Mailing-List' : pl.String,
                 'Body' : pl.String,
                 'Trailers': pl.List(pl.String),
                 'Code': pl.String}

BEFORE_SIGNED = "Body"
AFTER_SIGNED = "Code"
SIGNED_BLOCK = "Trailers"

FORCE_REPARSE = True #Always reparse every email on list. Otherwise parse only new emails.

KEYS_MASK = [
  "From",
  "To",
  "Cc",
  "Subject",
  "Date",
  "Message-ID",
  "In-Reply-To",
  "References",
  "X-Mailing-List",
  BEFORE_SIGNED,
  SIGNED_BLOCK,
  AFTER_SIGNED
]