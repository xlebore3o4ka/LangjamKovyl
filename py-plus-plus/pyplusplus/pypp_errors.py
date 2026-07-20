from enum import Enum

class PyppUnexpectedErrorType(str, Enum):
    """Enumeration of unexpected error types in pypp."""
    E01 = "E01: Expected a string value from {token_type}, but got {actual_type} instead on line {line_number}."

class PyppError(Exception):
    """Base class for all pypp errors."""
    message: str


class PyppUnexpectedError(PyppError):
    """Raised when an unexpected error occurs in pypp."""
    pass

class E01Error(PyppUnexpectedError):
    """
    Raised when an unexpected error of type E01 occurs in pypp.
    E01: Expected a string value from {token_type}, but got {actual_type} instead on line {line_number}.
    """
    def __init__(self, token_type: str, actual_type: str, line_number: int):
        self.message = PyppUnexpectedErrorType.E01.value.format(
            token_type=token_type,
            actual_type=actual_type,
            line_number=line_number
        )
        super().__init__(self.message)