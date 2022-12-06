"""Helpers for print note to easiler tracing the log"""
from typing import Any


def log_note(s: str) -> None:
    """Add note in log help to know the scenario"""
    print("\033[1;36mNOTE: " + s + " \033[0m")


class Section(object):
    """Add note for test section"""

    def __init__(self, section_name: str):
        self.section_name = section_name

    def __enter__(self) -> None:
        print("\033[1;36m" + "#" * (len(self.section_name) + 9) + " \033[0m")
        print("\033[1;36m# START: " + self.section_name + " \033[0m")

    def __exit__(self, _type: Any, _value: Any, traceback: Any) -> None:
        if not traceback:
            print("\033[1;36m| END:   " + self.section_name + " \033[0m")
            print("\033[1;36m" + "-" * (len(self.section_name) + 9) + " \033[0m")
