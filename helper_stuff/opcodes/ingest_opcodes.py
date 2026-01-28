#ingests the opcodes json object available at https://gbdev.io/gb-opcodes/optables/
import json

def to_upper_camel_case(s: str) -> str:
    return ''.join(word.capitalize() for word in s.replace('_', ' ').split())


class Operand:
    def __init__(self, operation_name: str, name: str, immediate: bool, increment: bool , decrement: bool):
        name = name.upper()
        self.name = name
        if name == "NC": self.name = "NotCarry"
        if name == "C" and operation_name in ["Jp", "Call", "Jr", "Ret" ]: self.name = "Carry"
        if name == "NZ": self.name = "NotZero"
        if name == "Z": self.name = "Zero"
        self.immediate = immediate
        self.increment = increment
        self.decrement = decrement

    def get_name(self, operation_name: str) -> str:
        if self.increment and self.name == "HL":  return self.name + "I"
        if self.decrement: return  self.name + "D"
        if self.name[0] == '$': return "Immediate("+self.name[1:] +")"
        if self.name.isdigit(): return "Immediate("+self.name + ")"
        if self.immediate: return self.name
        if (self.name == "C" or self.name == "A8") and operation_name == "Ldh": return "FF00OffsetBy" + self.name
        return self.name + "Pointer"

    def get_enum_variant(self, operation_name: str) -> str:
        return "Operand::" + self.get_name(operation_name)

class Operation:
    def __init__(
        self,
        opcode: str,
        name: str,
        bytes: int,
        cycles: list[int],
        operands: list[Operand],
        immediate: bool,
    ):
        self.opcode = opcode
        self.name = name
        self.bytes = bytes
        self.cycles = cycles
        self.operands = operands
        self.immediate = immediate

    def get_operands(self) -> str:
        operands = list()
        [operands.append(operand.get_enum_variant(self.name)) for operand in self.operands]
        return "&[" + ", ".join(operands) + "]"

    def get_instantiation(self) -> str:
        return "Instruction::new(OpCode::" + self.name + ", " + self.get_operands() + ", " + str(self.cycles[-1]) + ", " + str(self.bytes)  + ")"


with open("./opcodes.json", mode = 'r') as file:
    data = json.load(file)

unprefixed = list()
prefixed = list()

def create_list(list: list, specifier: str):
    for (opcode, operation) in data[specifier].items():

        list.append(
            Operation(
                opcode,
                to_upper_camel_case(operation["mnemonic"]),
                operation["bytes"],
                operation["cycles"],
                [Operand(operation["mnemonic"], operand["name"], operand["immediate"], operand.get("increment"), operand.get("decrement")) for operand in operation["operands"]],
                operation["immediate"],
            )
        )

create_list(unprefixed, "unprefixed")
create_list(prefixed, "cbprefixed")

def consolidate_illegal_ops():
    illegal_ops = set(
    ["IllegalD3", "IllegalDb", "IllegalDd", 
    "IllegalE3", "IllegalE4", "IllegalEb", 
    "IllegalEc", "IllegalEd", "IllegalF4", 
    "IllegalFc", "IllegalFd",]
    )
    for entry in unprefixed:
        if entry.name in illegal_ops:
            entry.name = "Illegal"


def get_table(specifier: str) -> str:
    if specifier == "unprefixed": this_list = unprefixed
    else: this_list = prefixed

    table = "pub const " + specifier.upper() + ": [Instruction;256] = [\n"
    for entry in this_list:
        table += entry.get_instantiation() + ",\n"

    table += '];'

    return table

consolidate_illegal_ops()

def get_opcode_enum() -> str:
    opcodes = set()
    [opcodes.add(instruction.name) for instruction in unprefixed]
    [opcodes.add(instruction.name) for instruction in prefixed]
    opcodes = sorted(opcodes)


    string: str = "pub enum OpCode {\n\t"
    string += ",\n\t".join(opcodes)
    string += "\n}\n"
    return string


def get_operand_enum() -> str:
    operands = set()
    for instruction in unprefixed:
        for operand in instruction.operands:
            operands.add(operand.get_name(instruction.name))
                
    operands = sorted(operands)


    string: str = "pub enum Operand {\n\t"
    string += ",\n\t".join(operands)
    string += "\n}\n"
    return string

# print(get_opcode_enum())
print(get_table("unprefixed"))
print(get_table("cbprefixed"))
# print(get_operand_enum())