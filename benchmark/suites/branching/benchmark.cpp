#include <cstdio>
#include <cstdint>

int main() {
    std::int32_t score = 85;
    std::int32_t attendance = 90;

    if (score >= 90) {
        std::puts("Excellent");
    } else if (score >= 75) {
        std::puts("Good");
    } else {
        std::puts("Needs Improvement");
    }

    if (attendance >= 80) {
        std::puts("Attendance Passed");
    } else {
        std::puts("Attendance Failed");
    }

    return 0;
}