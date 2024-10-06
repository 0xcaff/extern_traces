import struct
import sys

def patch_tls_segment(file_path):
    with open(file_path, 'rb+') as f:
        # Read SELF header
        if f.read(4) != b'\x4F\x15\x3D\x1D':
            raise ValueError("Invalid SELF magic")

        f.seek(0x18)
        segment_count = struct.unpack('<H', f.read(2))[0]

        # Calculate offset to ELF header
        elf_offset = 0x20 + segment_count * 32

        # Skip SELF segments
        f.seek(elf_offset)

        # Read ELF header
        if f.read(4) != b'\x7FELF':
            raise ValueError("Invalid ELF magic")

        if f.read(1) != b'\x02' or f.read(1) != b'\x01':
            raise ValueError("Not a 64-bit LSB ELF")

        f.seek(elf_offset + 0x20)
        ph_off = struct.unpack('<Q', f.read(8))[0]

        f.seek(elf_offset + 0x38)
        ph_count = struct.unpack('<H', f.read(2))[0]

        # Find and patch PT_TLS segment
        for i in range(ph_count):
            f.seek(elf_offset + ph_off + i * 56)
            p_type = struct.unpack('<I', f.read(4))[0]

            if p_type == 7:  # PT_TLS
                f.seek(elf_offset + ph_off + i * 56 + 0x28)
                old_mem_sz = struct.unpack('<Q', f.read(8))[0]
                f.seek(elf_offset + ph_off + i * 56 + 0x28)
                new_mem_sz = old_mem_sz + 32
                f.write(struct.pack('<Q', new_mem_sz))
                print(f"PT_TLS segment found and patched.")
                print(f"Old mem_sz: {old_mem_sz}")
                print(f"New mem_sz: {new_mem_sz}")
                return

        print("PT_TLS segment not found")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python patch_self.py <file_path> <new_mem_sz>")
        sys.exit(1)

    file_path = sys.argv[1]
    patch_tls_segment(file_path)
