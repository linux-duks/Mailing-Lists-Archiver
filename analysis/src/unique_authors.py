import polars as pl

# load the id mapping dataset from the anonymizer output
# for the normal dataset, a few changes must be made
df = pl.scan_parquet("/anonymousinput/__id_map_from/")

df = df.group_by(["__original_from", "from"]).agg(pl.col("list")).collect()

df.write_parquet("/app/results/unique_linux_authors.parquet")
