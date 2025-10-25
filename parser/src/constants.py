import polars as pl

PARQUET_COLS_SCHEMA = {'From' : pl.String,
                 'To': pl.String,#pl.List, # TODO: Update strings to lists of strings
                 'Cc': pl.String,#pl.List,
                 'Subject' : pl.String,
                 'Date' : pl.String, #pl.Datetime, # TODO: Update Datetime string to datetime 
                 'Message-ID': pl.String,
                 'In-Reply-To': pl.String,
                 'References': pl.String,#pl.List,
                 'X-Mailing-List' : pl.String,
                 'Body' : pl.String,
                 'Trailers': pl.String,#pl.List,
                 'Code': pl.String}

BEFORE_SIGNED = "Body"
AFTER_SIGNED = "Code"
SIGNED_BLOCK = "Trailers"

FORCE_REPARSE = False #Always reparse every email on list. Otherwise parse only new emails.

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