import re
from mlh_parser.constants import *


def set_value_dict(data: dict, key: str, value: str):
    if key in data:
        if isinstance(data[key], list):
            data[key].append(value)
        else:
            data[key] = [data[key], value]

    else:
        data[key] = value


def value_string_to_list(data: dict, keys: list) -> dict:
    for key in keys:
        value_changed = [item.strip() for item in data[key].split(",")]
        data[key] = value_changed

    return data


def value_list_to_string(data: dict) -> dict:
    for k, v in data.items():
        if isinstance(v, list):
            data[k] = ", ".join(map(str, v))

    return data


def filter_data(data: dict) -> dict:
    result_dict = {key: data.get(key, "") for key in KEYS_MASK}
    return result_dict


def parse_header_by_line(data: dict, line: str, current_key: str) -> str:
    exclude_inicial_caracters = (" ", "\t")

    if ":" in line and not (line.startswith(exclude_inicial_caracters)):
        key, value = line.split(":", 1)
        key = key.strip().lower()
        value = value.strip()

        set_value_dict(data, key, value)

        current_key = key
    else:
        if isinstance(data[current_key], list):
            data[current_key][-1] += " " + line.strip()
        else:
            data[current_key] += " " + line.strip()

    return current_key


# extract_attributions adapted from duks
# https://archive.softwareheritage.org/swh:1:cnt:23277d72e0a7a6f76db8542b10ab36e02a1e6006;origin=https://github.com/linux-duks/DUKS;visit=swh:1:snp:7539517b28d48726b43e4ef69332f3168b251aa0;anchor=swh:1:rev:de4af688e88757f0d496c7a16e331845a40d3f1c;path=/scripts/grpc_script.py;lines=82
def extract_attributions(commit_message) -> (list[dict], list[str]):
    """
    Parses a git commit message and extracts all personal attributions.

    Args:
        commit_message (str): The full git commit message.

    Returns:
        list: A list of dictionaries, where each dictionary contains
              'type' (e.g., 'Signed-off-by'), 'name', and 'email' for each attribution found.
    """
    attributions = []
    # This regex looks for lines starting with "Word-by:"
    # followed by a name (can contain various characters), and then an email in angle brackets.
    # It captures the "Word-by" type, the name, and the email separately.
    # The pattern is compiled with MULTILINE to match '^' at the start of each line,
    # and IGNORECASE to match "Signed-off-by", "signed-off-by", etc.
    pattern = re.compile(
        r"^(?P<type>[a-zA-Z\-]+-by):[ \t]*(?P<name>[^<\n]+?)[ \t]*<(?P<email>[^>\n]+)>",
        re.MULTILINE | re.IGNORECASE,
    )

    # Use finditer to get all non-overlapping matches
    for match in pattern.finditer(commit_message):
        attributions.append(
            {
                "attribution": match.group(
                    "type"
                ).strip(),  # Extract the attribution type (e.g., 'Signed-off-by')
                "identification": match.group("name").strip()
                + f""" <{
                    match.group("email").strip()
                }>""",  # Extract the name (e.g., 'Author Name')
            }
        )
    return attributions


signed_block_regex = r"^\S+-By: [\S\s]* <\S+@\S+>"


def parse_body_by_line(data: dict, line: str, body_state: dict):
    # TODO: fix code detection
    if body_state["body_state"] == "before_signed" and re.match(
        signed_block_regex, line, re.IGNORECASE
    ):
        body_state["body_state"] = "signed_block"

    elif body_state["body_state"] == "signed_block":
        if not re.match(r"^\S+-By: [\S\s]* <\S+@\S+>", line, re.IGNORECASE):
            body_state["body_state"] = "after_signed"
            set_value_dict(data, AFTER_SIGNED, line)

    elif body_state["body_state"] == "before_signed":
        if not BEFORE_SIGNED in data:
            data.setdefault(BEFORE_SIGNED, "")
        data[BEFORE_SIGNED] += line + "\n"

    elif body_state["body_state"] == "after_signed":
        if not AFTER_SIGNED in data:
            data.setdefault(AFTER_SIGNED, "")
        data[AFTER_SIGNED] += line + "\n"


def parse_email_txt_to_dict(text: str) -> object:
    data = {}
    current_key = None
    lines = text.splitlines()
    parser_state = {"is_body": False, "body_state": "before_signed"}
    data["raw_body"] = ""

    raw_body_lines = []

    # se a linha começar com \n -> começa o corpo do email
    for line in lines:
        if not parser_state["is_body"] and not line.strip():
            parser_state["is_body"] = True
            continue

        if parser_state["is_body"]:
            raw_body_lines.append(line)
            parse_body_by_line(data, line, parser_state)
        else:
            current_key = parse_header_by_line(data, line, current_key)

    # read trailers from the raw body
    data["raw_body"] = "\n".join(raw_body_lines)
    attributions = extract_attributions(data["raw_body"])
    data[SIGNED_BLOCK] = attributions

    data = filter_data(data)

    return data
