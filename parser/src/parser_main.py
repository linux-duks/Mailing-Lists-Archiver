import io
import os
import polars as pl

from parser_algorithm import parse_email_txt_to_dict
from constants import PARQUET_COLS_SCHEMA

INPUT_DIR_PATH = os.environ["INPUT_DIR"]
OUTPUT_DIR_PATH = os.environ["OUTPUT_DIR"]
PARQUET_DIR_PATH = OUTPUT_DIR_PATH + "/parsed"
PARQUET_FILE_NAME = "list_data.parquet"

def parse_mail_at(mailing_list):
    """
    Parses the emails from a single specified list,
    to be found in INPUT_DIR_PATH/mailing_list
    """

    list_input_path = INPUT_DIR_PATH + "/" + mailing_list
    list_output_path = OUTPUT_DIR_PATH + "/" + mailing_list
    success_output_path = PARQUET_DIR_PATH + "/list=" + mailing_list
    parquet_path = success_output_path + "/" + PARQUET_FILE_NAME
    error_output_path = list_output_path + "/errors"

    all_parsed = pl.DataFrame(schema=PARQUET_COLS_SCHEMA)

    if not os.path.isdir(list_output_path):
        print("First parse of list", mailing_list)

        if not os.path.isdir(PARQUET_DIR_PATH):
          os.mkdir(PARQUET_DIR_PATH)

        os.mkdir(list_output_path)
        os.mkdir(success_output_path)
        os.mkdir(error_output_path)

        all_parsed = pl.DataFrame(schema=PARQUET_COLS_SCHEMA)
    else:
        all_parsed = pl.read_parquet(parquet_path)
    

    all_emails = os.listdir(list_input_path)
    all_emails.remove("__last_article_number")

    for email_name in all_emails:
        email_path = list_input_path + "/" + email_name
        email_file = io.open(email_path, mode="r", encoding="utf-8")

        try:
            email_as_dict = parse_email_txt_to_dict(email_file.read())
        except Exception as parsing_error:

            to_save = email_file.read() 
            to_save += '\n='*30 + " Exception:"
            to_save += str(parsing_error)

            print("Error when parsing file", email_name, "of list", mailing_list)
            print(parsing_error)

            with open(error_output_path + '/' + email_name,"w",encoding="utf-8") as error_output_file:
                error_output_file.write(to_save)

            email_file.close()
            continue

        email_as_df = pl.DataFrame(email_as_dict,schema=PARQUET_COLS_SCHEMA)

        all_parsed.extend(email_as_df)

        all_parsed.write_parquet(parquet_path)
        email_file.close()

        break
        

def main():
    for mailing_list in os.listdir(INPUT_DIR_PATH):
        parse_mail_at(mailing_list)

if __name__ == "__main__":
    main()
