from pyspark.sql.functions import udf
from pyspark.sql.types import BooleanType, StringType, IntegerType
import ipaddress


@udf(returnType=BooleanType())
def cidr_match(ip_address, cidr_range):
    try:
        # Parse the IP address
        ip = ipaddress.ip_address(ip_address)

        # Parse the CIDR range
        network = ipaddress.ip_network(cidr_range, strict=False)

        # Check if the IP is in the network
        return ip in network
    except ValueError:
        # If there's an error parsing the IP or CIDR, return False
        return False


@udf(returnType=StringType())
def ipmask(mask, ip):
    try:
        # Convert mask to integer
        mask_int = int(mask)

        # Create an IPv4Network object
        network = ipaddress.IPv4Network(f"{ip}/{mask_int}", strict=False)

        # Return the network address (masked IP)
        return str(network.network_address)
    except ValueError:
        # Return None if there's an error (invalid IP or mask)
        return None


@udf(returnType=StringType())
def printf(format_str, args):
    return format_str % args


@udf(returnType=IntegerType())
def tonumber(value, base=10):
    try:
        return int(value, base)
    except (ValueError, TypeError):
        return None


@udf(returnType=StringType())
def tostring(value, format: str | None = None):
    # "binary", "hex", "commas", "duration"
    match (value, format):
        case (bool(), None):
            return "True" if value else "False"
        case (int() | float(), None):
            return str(value)
        case (int(), "binary"):
            return bin(value)[2:]
        case (int(), "hex"):
            return hex(value)[2:]
        case (int() | float(), "commas"):
            return "{:,}".format(value)
        case (int(), "duration"):
            hours, remainder = divmod(value, 3600)
            minutes, seconds = divmod(remainder, 60)
            return f"{hours:02d}:{minutes:02d}:{seconds:02d}"
        case _:
            raise TypeError(f"Unsupported value+format pair: ({value}, {format})")
