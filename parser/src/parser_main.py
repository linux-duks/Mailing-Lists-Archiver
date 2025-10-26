import io
import os
import re
import polars as pl
from datetime import datetime

from parser_algorithm import parse_email_txt_to_dict
from constants import PARQUET_COLS_SCHEMA, FORCE_REPARSE

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
    else:
        if FORCE_REPARSE:
            remove_previous_errors(error_output_path)
        all_parsed = pl.read_parquet(parquet_path)
    
    all_emails = os.listdir(list_input_path)
    all_emails.remove("__last_article_number")
    if "errors.md" in all_emails:
        all_emails.remove("errors.md")
    all_parsed = all_parsed.with_row_index()

    for email_name in all_emails:
        email_path = list_input_path + "/" + email_name
        email_file = io.open(email_path, mode="r", encoding="utf-8")

        email_id = get_email_id(email_file)
        previous_index = email_previously_parsed(all_parsed,email_id)

        #  Check whether email was parsed previously, so it won't even
        # be parsed again if FORCE_REPARSE is set to False.
        if previous_index is not None and not FORCE_REPARSE:
            email_file.close()
            continue

        try:
            email_as_dict = parse_email_txt_to_dict(email_file.read())
            email_as_dict = post_process_parsed_mail(email_as_dict)
        except Exception as parsing_error:
            save_unsuccessful_parse(email_file, parsing_error, email_name, mailing_list, error_output_path)
            continue

        email_as_df = pl.DataFrame(email_as_dict,schema=PARQUET_COLS_SCHEMA)
        email_as_df = email_as_df.with_columns( # Let's keep our datetimes naive
            pl.col("date").dt.replace_time_zone(None)
        )

        email_as_df = email_as_df.with_row_index()

        if previous_index is not None: # Note that, necessarily, FORCE_REPARSE == True
            
            all_parsed = pl.concat([
                all_parsed.slice(0,previous_index),
                email_as_df,
                all_parsed.slice(previous_index+1)
            ])
            
        else:
            all_parsed.extend(email_as_df) # Simply adds to end of DF

        email_file.close()

    all_parsed = all_parsed.drop("index")
    all_parsed.write_parquet(parquet_path)
    #print(all_parsed)

def post_process_parsed_mail(email_as_dict: dict):
    """
    Post-processes dict containing email fields, parsing
    multiple valued fields and other non Str fields.
    """ 

    # TODO: Anonymize everything here
    
    email_as_dict["cc"] = email_as_dict["cc"].split(',')
    email_as_dict["references"] = email_as_dict["references"].split(' ')
    email_as_dict["trailers"] = email_as_dict["trailers"].split(',')

    old_date_time = email_as_dict["date"]

    try:
        new_date_time = datetime.strptime(old_date_time, "%a, %d %b %Y %X %z")
    except Exception:
        new_date_time = datetime.strptime(old_date_time[:-6].strip(), "%a, %d %b %Y %X %z")

    if isinstance(email_as_dict["subject"],list):
        email_as_dict["subject"] = email_as_dict["subject"][0]

    email_as_dict["date"] = new_date_time

    for dict_key in email_as_dict:
        email_as_dict[dict_key] = [email_as_dict[dict_key]]

    return email_as_dict

def get_email_id(email_file) -> str:
    """
    Retrieves the email Message-ID.
    """

    for line in email_file.readlines():
        if re.match(r"^Message-ID:", line,re.IGNORECASE):
            message_id = line[len("Message-ID:"):].strip()
            email_file.seek(0,os.SEEK_SET)
            return message_id
        
    email_file.seek(0,os.SEEK_SET) # Return to the beginning of file stream
    
    raise Exception("Found email with no Message-ID field for file " + email_file.name)

def email_previously_parsed(all_parsed,email_id) -> int | None:
    """
    Checks whether the given email message id corresponds
    to a email saved in the archive. If that's the case, 
    returns the dataframe row where the email is stored.
    Otherwise, returns None.
    """
    
    filter_res = all_parsed.filter(pl.col('message-id') == email_id)

    if filter_res.shape[0] == 0:
        return None
    elif filter_res.shape[0] > 1:
        raise Exception("Message-ID conflict on parquet databse for id " + email_id)

    return filter_res[0,'index']

def save_unsuccessful_parse(email_file, parsing_error, email_name, mailing_list, error_output_path):
    """
    Saves information on unsuccessful email parse. Both original email content and
    exception information are stored in the directory at <error_output_path>, in
    a file with the same name as the original .eml file.
    """
    email_file.seek(0,os.SEEK_SET) # Return to the beginning of file stream
    
    to_save = email_file.read()
    to_save += '\n' + '='*30 + " Exception:\n"
    to_save += str(parsing_error)

    print("Error when parsing file", email_name, "of list", mailing_list)
    print(parsing_error)

    with open(error_output_path + '/' + email_name,"w",encoding="utf-8") as error_output_file:
        error_output_file.write(to_save)

    email_file.close()

def remove_previous_errors(errors_dir_path):
    """
    Removes every file from the directory at the path given.
    """

    for error_file_name in os.listdir(errors_dir_path):
        os.remove(errors_dir_path + '/' + error_file_name)

def main():
    for mailing_list in os.listdir(INPUT_DIR_PATH):
        parse_mail_at(mailing_list)

if __name__ == "__main__":
    main()
