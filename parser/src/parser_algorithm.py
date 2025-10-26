import re
from constants import *


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
    key = key.strip()
    value = value.strip()

    set_value_dict(data, key, value)

    current_key = key 
  else:              
    if isinstance(data[current_key], list):
      data[current_key][-1] += " " + line.strip()
    else:
      data[current_key] += " " + line.strip()
  
  return current_key


def parse_body_by_line(data: dict, line: str, body_state: dict):

  if body_state["body_state"] == "before_signed" and re.match(r"^\S+-By: [\S\s]* <\S+@\S+>", line, re.IGNORECASE):
    body_state["body_state"] = "signed_block"
    data[SIGNED_BLOCK] = line
  
  elif body_state["body_state"] == "signed_block":
    if re.match(r"^\S+-By: [\S\s]* <\S+@\S+>", line, re.IGNORECASE):
      data[SIGNED_BLOCK] += ',' + line 
    else:
      body_state["body_state"] = "after_signed"
      set_value_dict(data,AFTER_SIGNED,line)
  
  elif body_state["body_state"] == "before_signed":
    if not BEFORE_SIGNED in data: 
      data.setdefault(BEFORE_SIGNED, "")
    data[BEFORE_SIGNED] += line  + '\n'

  elif body_state["body_state"] == "after_signed":
    if not AFTER_SIGNED in data: 
      data.setdefault(AFTER_SIGNED, "")
    data[AFTER_SIGNED] += line  + '\n'


def parse_email_txt_to_dict(text: str) -> object:
  data = {}
  current_key = None
  lines = text.splitlines()
  parser_state = {
    "is_body": False,
    "body_state": "before_signed"
  }

  # se a linha começar com \n -> começa o corpo do email
  for line in lines:
    if not parser_state["is_body"] and not line.strip():
      parser_state["is_body"] = True
      continue
    
    if parser_state["is_body"]:
      parse_body_by_line(data, line, parser_state)

    else:
      current_key = parse_header_by_line(data, line, current_key)
          
  data = filter_data(data)

  return data