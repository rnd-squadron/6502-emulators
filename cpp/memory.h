#include <cstddef>
#include <cstdint>
#include <array>
#include <vector>
#include <stdexcept>

template<std::size_t L>
class RAM {
public:
  void store(std::size_t, std::uint8_t);
  void store(std::size_t, std::uint16_t);
  std::uint8_t load8(std::size_t);
  std::uint16_t load16(std::size_t);
private:
  void check_for_bounds(std::size_t addr);
  
  std::array<std::uint8_t, L> mem;
};

template<std::size_t L>
void RAM<L>::store(std::size_t addr, std::uint8_t v) {
  check_for_bounds(addr);
  mem[addr] = v;
}

template<std::size_t L>
void RAM<L>::store(std::size_t addr, std::uint16_t v) {
  check_for_bounds(addr + 1);
  
  // Assuming that the value is big-endian.
  // Firstly extract low byte of the value.
  std::uint8_t b = v;
  mem[addr] = b;
  
  // Then high byte of the value.
  b = v >> 8;
  mem[addr + 1] = b;
}

template<std::size_t L>
std::uint8_t RAM<L>::load8(std::size_t addr) {
  check_for_bounds(addr);
  return mem[addr];
}

template<std::size_t L>
std::uint16_t RAM<L>::load16(std::size_t addr) {
  check_for_bounds(addr + 1);

  // Assuming that the returned value is big-endian.
  std::uint16_t tmp = mem[addr + 1];
  return (tmp << 8) | mem[addr];
}

template<std::size_t L>
void RAM<L>::check_for_bounds(std::size_t addr) {
  if (addr >= mem.size()) {
    throw std::out_of_range("address out of memory");
  }
}
