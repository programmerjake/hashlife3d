#define VK_DEFINE_NON_DISPATCHABLE_HANDLE(x) typedef uint64_t x;
#define VK_NO_PROTOTYPES
#include "vulkan/vulkan.h"