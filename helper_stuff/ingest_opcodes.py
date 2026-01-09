import json

def to_upper_camel_case(s: str) -> str:
    return ''.join(word.capitalize() for word in s.replace('_', ' ').split())


class Operand:
    def __init__(self,  name: str, immediate: bool, increment: bool , decrement: bool):
        self.name = name.upper()
        self.immediate = immediate
        self.increment = increment
        self.decrement = decrement

    def get_name(self) -> str:
        if self.increment:
            if self.name == "SP": return self.name+"AndE8"
            else: return self.name + "I"
        if self.decrement: return  self.name + "D"
        if self.name[0] == '$': return "Immediate("+self.name[1:] +")"
        if self.name.isdigit(): return "Immediate("+self.name + ")"
        if self.immediate: return self.name
        if self.name == "C" or self.name == "A8": return "FF00OffsetBy" + self.name[0]
        return self.name + "Pointer"

    def get_enum_variant(self) -> str:
        return "Operand::" + self.get_name()

class Flags:
    def __init__(
            self, z: str, n: str, h: str, c: str
            ):
        self.z = z
        self.n = n
        self.h = h
        self.c = c
    
    def get_z_choice(self) -> str:
        if self.z == "-": return "None"
        if self.z == "0": return "Some(FlagCheck::SetToValue(0))"
        if self.z == "1": return "Some(FlagCheck::SetToValue(1))"
        if self.z == "Z": return "Some(FlagCheck::Check)"
        return "MISSED CASE ON Z"
    def get_n_choice(self) -> str:
        if self.n == "-": return "None"
        if self.n == "N": return "Some(FlagCheck::Check)"
        if self.n == "0": return "Some(FlagCheck::SetToValue(0))"
        if self.n == "1": return "Some(FlagCheck::SetToValue(1))"
        return "MISSED CASE ON N"
    def get_h_choice(self) -> str:
        if self.h == "-": return "None"
        if self.h == "0": return "Some(FlagCheck::SetToValue(0))"
        if self.h == "1": return "Some(FlagCheck::SetToValue(1))"
        if self.h == "H": return "Some(FlagCheck::CheckOverflowAtBit(3))" #TODO: Update this to account for 2 byte operands
        return "MISSED CASE ON H"
    def get_c_choice(self) -> str:
        if self.c == "-": return "None"
        if self.c == "0": return "Some(FlagCheck::SetToValue(0))"
        if self.c == "1": return "Some(FlagCheck::SetToValue(1))"
        if self.c == "C": return "Some(FlagCheck::CheckOverflowAtBit(7))" #TODO: Update this to account for 2 byte operands
        return "MISSED CASE ON C"

    def get_instantiation(self) -> str:
        return "FlagChecks::new(" + self.get_z_choice() + ", " + self.get_n_choice() + ", " +self.get_h_choice() + ", " +self.get_c_choice() +")"

class Operation:
    def __init__(
        self,
        opcode: str,
        name: str,
        bytes: int,
        cycles: list[int],
        operands: list[Operand],
        immediate: bool,
        flags: Flags
    ):
        if ((name == "Jp" or name == "Jr" or name == "Call" ) and len(operands) == 2 )or (name == "Ret" and len(operands) == 1):
            name += "Conditional"
        self.opcode = opcode
        self.name = name
        self.bytes = bytes
        self.cycles = cycles
        self.operands = operands
        self.immediate = immediate
        self.flags = flags

    def get_operands(self) -> str:
        operands = list()
        [operands.append(operand.get_enum_variant()) for operand in self.operands]
        return "&[" + ", ".join(operands) + "]"

    def get_instantiation(self) -> str:
        return "Instruction::new(OpCode::" + self.name + ", " + self.get_operands() + ", " + str(self.cycles[0]) + ", " + str(self.bytes) + ", " + self.flags.get_instantiation() + ")"


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
                [Operand(operand["name"], operand["immediate"], operand.get("increment"), operand.get("decrement")) for operand in operation["operands"]],
                operation["immediate"],
                Flags(
                    operation["flags"]["Z"],
                    operation["flags"]["N"],
                    operation["flags"]["H"],
                    operation["flags"]["C"],
                ),
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
            operands.add(operand.get_name())
                
    operands = sorted(operands)


    string: str = "pub enum Operand {\n\t"
    string += ",\n\t".join(operands)
    string += "\n}\n"
    return string

# print(get_opcode_enum())
print(get_table("unprefixed"))
print(get_table("cbprefixed"))
# print(get_operand_enum())