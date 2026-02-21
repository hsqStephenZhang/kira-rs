int main() {
    int a = 0;
    int b = 0;
    int c = 0;
    int d = 0;
    a = 1;
    if (a) {
    } else {
        b = 1;
    }
    if (a || b) {
        b = b + 1;
    }
    c = 1;
    if (c) { d = 1; }
    if (d) {
        d = d + 1;
    }
    return b == 1 && d == 2;
}
