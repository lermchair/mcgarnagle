import base64
import enum
from cryptography.fernet import Fernet


class GateType(enum.Enum):
    AND = "AND"
    NOT = "NOT"
    OR = "OR"
    XOR = "XOR"


def generate_keys(delta: bytes) -> tuple[bytes, bytes]:
    zero = Fernet.generate_key()
    return zero, bytes_xor(zero, delta)


def encrypt(key: bytes, data: bytes) -> bytes:
    return Fernet(key).encrypt(data)


def decrypt(key: bytes, data: bytes) -> bytes:
    return Fernet(key).decrypt(data)


def bytes_xor(a: bytes, b: bytes) -> bytes:
    key1_bytes = base64.urlsafe_b64decode(a)
    key2_bytes = base64.urlsafe_b64decode(b)
    xor_result = bytes(a ^ b for a, b in zip(key1_bytes, key2_bytes))
    return base64.urlsafe_b64encode(xor_result)
