from freezegun import freeze_time
import io
import pytest
import datetime
from mlh_parser.email_reader import decode_mail, get_body, get_headers

from .helpers import list_files_with_extension
# from mlh_parser.parser import resolve_dates

directory = "./date_cases/"
email_files = list_files_with_extension(directory, ".eml")

# real_mail_files = [map_to_files(email_f) for email_f in email_files]


@freeze_time("2025-12-21")
# @pytest.mark.parametrize("email_file, code_file, trailers_file", real_mail_files)
@pytest.mark.parametrize("email_file", email_files)
def test_corret_email(email_file) -> None:
    mail_bytes = io.open(email_file, mode="rb").read()

    msg = decode_mail(mail_bytes)
    charset = msg.get_content_charset()
    print("charset", charset)

    print("items ", get_headers(msg))

    # print(mail)

    # print(mail_text)
    # code = eval(io.open(code_file, mode="r", encoding="utf-8").read())
    # trailers = eval(io.open(trailers_file, mode="r", encoding="utf-8").read())
    assert False


#     attr = extract_attributions(input_example_mail)
#     assert len(attr) == 1
#     assert attr == [
#         {
#             "attribution": "Signed-off-by",
#             "identification": "Example Contributor <example@contributor.com>",
#         }
#     ]
