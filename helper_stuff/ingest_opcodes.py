import json

class Operand:
    def __init__(self,  name: str, immediate: bool, bytes: int = 0):
        self.name = name
        self.immediate = immediate
        self.bytes = bytes

class Flags:
    def __init__(
            self, z: str, n: str, h: str, c: str
            ):
        self.z = z
        self.n = n
        self.h = h
        self.c = c
    
    def get_instantiation(self) -> str:
        return "todo"

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
        self.opcode = opcode
        self.name = name
        self.bytes = bytes
        self.cycles = cycles
        self.operands = operands
        self.immediate = immediate
        self.flags = flags

    def get_instantiation(self) -> str:
        return "Instruction::new(" + self.name + ", " + str(self.cycles[0]) + ", " + str(self.bytes) + ", " + self.flags.get_instantiation() + ")"

with open("./opcodes.json", mode = 'r') as file:
    data = json.load(file)

unprefixed = list()

for (opcode, operation) in data["unprefixed"].items():

    unprefixed.append(
        Operation(
            opcode,
            operation["mnemonic"],
            operation["bytes"],
            operation["cycles"],
            list(),
            operation["immediate"],
            Flags(
                operation["flags"]["Z"],
                operation["flags"]["N"],
                operation["flags"]["H"],
                operation["flags"]["C"],
            ),
        )
    )



def get_unprefixed_table(unprefixed: list[Operation]) -> str:
    table = "pub const UNPREFIXED: &[Instruction] = {\n"

    for entry in unprefixed:
        table += "\t" + entry.get_instantiation() + ",\n"

    table += '\n}'

    return table



print(get_unprefixed_table(unprefixed))