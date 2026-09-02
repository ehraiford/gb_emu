// Cycle-level trace dumper for SameBoy's core, used as the reference side of a
// divergence comparison against gb_emu. One line per instruction retire:
//   <m_cycle> <pc> <af> <bc> <de> <hl> <sp> <ly> <stat> <div> <tima> <if> <ie>
// The cumulative M-cycle count is the whole point: an instruction that takes the
// wrong number of cycles shows up as a drift in column 1 long before any register
// looks wrong.
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include "Core/gb.h"

// Declared in Core/memory.h, which is gated behind GB_INTERNAL. The symbol is
// exported all the same, so declaring it here keeps this file off the internal headers.
extern uint8_t GB_safe_read_memory(GB_gameboy_t *gb, uint16_t addr);

// The display writes into this every frame. Without it the core dereferences a null output
// pointer the moment the first frame completes, which is why a short trace looks fine and a
// long one segfaults. Sized for the largest model rather than DMG's 160x144.
static uint32_t framebuffer[256 * 224];

static uint32_t rgb_encode(GB_gameboy_t *gb, uint8_t r, uint8_t g, uint8_t b)
{
    (void)gb;
    return ((uint32_t)r << 16) | ((uint32_t)g << 8) | b;
}

static long read_file(const char *path, uint8_t **out)
{
    FILE *f = fopen(path, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    *out = malloc(size);
    if (fread(*out, 1, size, f) != (size_t)size) { fclose(f); return -1; }
    fclose(f);
    return size;
}

int main(int argc, char **argv)
{
    if (argc < 5) {
        fprintf(stderr, "usage: tracer <rom> <bootrom> <max_instructions> <out.trace>\n");
        return 2;
    }
    const char *rom_path = argv[1];
    const char *boot_path = argv[2];
    long max_instructions = atol(argv[3]);
    // The boot ROM runs for roughly a million M-cycles before handing off. Suppressing output
    // until past it keeps trace files manageable when the interesting code is in the cartridge.
    unsigned long long skip_m_cycles = argc > 5 ? strtoull(argv[5], NULL, 10) : 0;

    uint8_t *boot = NULL;
    long boot_size = read_file(boot_path, &boot);
    if (boot_size < 0) { fprintf(stderr, "could not read boot rom %s\n", boot_path); return 1; }

    uint8_t *rom = NULL;
    long rom_size = read_file(rom_path, &rom);
    if (rom_size < 0) { fprintf(stderr, "could not read rom %s\n", rom_path); return 1; }

    GB_gameboy_t *gb = GB_init(GB_alloc(), GB_MODEL_DMG_B);
    GB_set_rgb_encode_callback(gb, rgb_encode);
    GB_set_pixels_output(gb, framebuffer);
    GB_load_boot_rom_from_buffer(gb, boot, boot_size);
    GB_load_rom_from_buffer(gb, rom, rom_size);

    FILE *out = fopen(argv[4], "wb");
    if (!out) { fprintf(stderr, "could not open %s for writing\n", argv[4]); return 1; }
    setvbuf(out, NULL, _IOFBF, 1 << 20);

    GB_registers_t *r = GB_get_registers(gb);
    uint64_t ticks_8mhz = 0;

    for (long i = 0; i < max_instructions; i++) {
        unsigned long long m_cycle = ticks_8mhz / 8;
        if (m_cycle < skip_m_cycles) {
            ticks_8mhz += GB_run(gb);
            continue;
        }
        // State is sampled *before* the instruction runs, so pc names the instruction
        // that is about to execute rather than the one that just finished.
        fprintf(out, "%llu %04X %04X %04X %04X %04X %04X %02X %02X %02X %02X %02X %02X\n",
                m_cycle,
                r->pc, r->af, r->bc, r->de, r->hl, r->sp,
                GB_safe_read_memory(gb, 0xFF44),  // LY
                GB_safe_read_memory(gb, 0xFF41),  // STAT
                GB_safe_read_memory(gb, 0xFF04),  // DIV
                GB_safe_read_memory(gb, 0xFF05),  // TIMA
                GB_safe_read_memory(gb, 0xFF0F),  // IF
                GB_safe_read_memory(gb, 0xFFFF)); // IE
        ticks_8mhz += GB_run(gb);
    }

    fclose(out);
    GB_free(gb);
    return 0;
}
