#include <cstdlib>
#include <cstring>

#include "Luau/BytecodeBuilder.h"
#include "Luau/BytecodeUtils.h"
#include "Luau/Compiler.h"

namespace {

struct Encoder : Luau::BytecodeEncoder {
    void encode(uint32_t* data, size_t count) override {
        for (size_t i = 0; i < count;) {
            uint8_t op = LUAU_INSN_OP(data[i]);
            data[i] = uint8_t(op * 227) | (data[i] & ~0xffu);
            i += Luau::getOpLength(LuauOpcode(op));
        }
    }
};

}

extern "C" char* sunlight_compile(const char* source, size_t size, size_t* out) {
    static const char* mutable_globals[] = {
        "Game", "Workspace", "game", "plugin", "script", "shared", "workspace", nullptr,
    };
    Luau::CompileOptions options;
    options.optimizationLevel = 1;
    options.debugLevel = 1;
    options.vectorLib = "Vector3";
    options.vectorCtor = "new";
    options.vectorType = "Vector3";
    options.mutableGlobals = mutable_globals;

    Encoder encoder;
    std::string bytecode = Luau::compile(std::string(source, size), options, {}, &encoder);
    *out = bytecode.size();
    return static_cast<char*>(std::memcpy(std::malloc(*out), bytecode.data(), *out));
}
