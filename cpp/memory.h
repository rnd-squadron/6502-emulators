#include <cstddef>
#include <cstdint>
#include <array>
#include <vector>
#include <stdexcept>

template<std::size_t Size>
class RAM {
public:
  void store(std::size_t, std::uint8_t);
  void store(std::size_t, std::uint16_t);
  std::uint8_t load8(std::size_t);
  std::uint16_t load16(std::size_t);
private:
  void check_for_bounds(std::size_t addr);
  
  std::array<std::uint8_t, Size> mem;
};

template<std::size_t Size>
void RAM<Size>::store(std::size_t addr, std::uint8_t v) {
  check_for_bounds(addr);
  mem[addr] = v;
}

template<std::size_t Size>
void RAM<Size>::store(std::size_t addr, std::uint16_t v) {
  check_for_bounds(addr + 1);
  
  // Assuming that the value is big-endian.
  // Firstly extract low byte of the value.
  mem[addr] = v;
  
  // Then high byte of the value.
  mem[addr + 1] = v >> 8;
}

template<std::size_t Size>
std::uint8_t RAM<Size>::load8(std::size_t addr) {
  check_for_bounds(addr);
  return mem[addr];
}

template<std::size_t Size>
std::uint16_t RAM<Size>::load16(std::size_t addr) {
  check_for_bounds(addr + 1);

  // Assuming that the returned value is big-endian.
  return static_cast<uint16_t>(mem[addr + 1]) << 8 | mem[addr];
}

template<std::size_t Size>
void RAM<Size>::check_for_bounds(std::size_t addr) {
  if (addr >= mem.size()) {
    throw std::out_of_range("address out of memory");
  }
}
