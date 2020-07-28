#! /usr/bin/python

import re

match_regex = re.compile(r"match\s+\{")
else_regex = re.compile(r"\}\s+else\s+\{")

name_regex = re.compile(r"""(r"([^"]+)")""")
word = re.compile(r"[a-z]+")


stat = False
output = ""

with open('src/simpla.lalrpop') as file:
    for line in file.readlines():
        if match_regex.match(line):
            stat = True
        elif else_regex.match(line):
            stat = False

        if stat:
            line = line.strip()
            if match := name_regex.match(line):
                regex = match[1]
                name = match[2]
                if word.match(name):
                    terminal_name = name.capitalize() + "KW"
                else:
                    terminal_name = input(f"Name for {regex} ")
                    terminal_name += "Punct"
                line = f"{terminal_name} = <{regex}>;\n"
                output += line
                
with open('terminals.txt', "w") as file:
    print(output, file=file)

