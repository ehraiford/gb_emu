import json

def to_upper_camel_case(s: str) -> str:
    return ''.join(word.capitalize() for word in s.replace('_', ' ').split())

class Operand:
    def __init__(self, operation_mnemonic: str, name: str, immediate: bool, increment: bool, decrement: bool):
        mnemonic = operation_mnemonic.lower()
        name = name.upper()
        self.name = name
        
        if name == "NC": self.name = "NotCarry"
        if name == "C" and mnemonic in ["jp", "call", "jr", "ret"]: 
            self.name = "Carry"
        if name == "NZ": self.name = "NotZero"
        if name == "Z": self.name = "Zero"

        self.immediate = immediate
        self.increment = increment
        self.decrement = decrement

    def get_name(self, operation_name: str) -> str:
        if self.increment and self.name == "HL": return self.name + "I"
        if self.decrement: return self.name + "D"
        
        if self.name.startswith('$'): 
            return f"Immediate(0x{self.name[1:]})"
        if self.name.isdigit(): 
            return f"Immediate(0x{self.name})"
            
        if self.immediate: return self.name
        
        if (self.name == "C" or self.name == "A8") and operation_name.lower() == "ldh": 
            return "FF00OffsetBy" + self.name
            
        return self.name + "Pointer"

    def get_enum_variant(self, operation_name: str) -> str:
        return "Operand::" + self.get_name(operation_name)

class Operation:
    def __init__(self, opcode: str, name: str, mnemonic: str, bytes_count: int, cycles: list[int], operands: list[Operand]):
        self.opcode = opcode
        self.name = name
        self.mnemonic = mnemonic
        self.bytes = bytes_count
        self.cycles = cycles
        self.operands = operands

    def get_operands(self) -> str:
        operands = [operand.get_enum_variant(self.name) for operand in self.operands]
        return "&[" + ", ".join(operands) + "]"

    def get_instantiation(self) -> str:
        return f"Instruction::new(OpCode::{self.name}, {self.get_operands()}, {str(int(self.cycles[-1] / 4))}, {self.bytes})"

def create_list(target_list: list, specifier: str, data: dict):
    for opcode_hex, operation in data[specifier].items():
        mnemonic = operation["mnemonic"]
        target_list.append(
            Operation(
                opcode_hex,
                to_upper_camel_case(mnemonic),
                mnemonic,
                operation["bytes"],
                operation["cycles"],
                [Operand(
                    mnemonic, 
                    op["name"], 
                    op["immediate"], 
                    op.get("increment", False), 
                    op.get("decrement", False)
                ) for op in operation.get("operands", [])]
            )
        )

with open("./opcodes.json", mode='r') as file:
    data = json.load(file)

unprefixed = []
prefixed = []

create_list(unprefixed, "unprefixed", data)
create_list(prefixed, "cbprefixed", data)

def consolidate_illegal_ops():
    illegal_ops = set([
        "IllegalD3", "IllegalDb", "IllegalDd", 
        "IllegalE3", "IllegalE4", "IllegalEb", 
        "IllegalEc", "IllegalEd", "IllegalF4", 
        "IllegalFc", "IllegalFd",
    ])
    for entry in unprefixed:
        if entry.name in illegal_ops:
            entry.name = "Illegal"

def get_table(specifier: str) -> str:
    if specifier == "unprefixed": this_list = unprefixed
    else: this_list = prefixed

    table = "pub const " + specifier.upper() + ": [Instruction; 256] = [\n"
    sorted_list = sorted(this_list, key=lambda op: int(op.opcode, 16))
    
    
    for entry in sorted_list:
        table += "    " + entry.get_instantiation() + ",\n"

    table += '];'
    return table

consolidate_illegal_ops()

print(get_table("unprefixed"))
print(get_table("cbprefixed"))