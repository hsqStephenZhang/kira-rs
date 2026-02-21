int f(int x) {
    return x + 8;
}
int main() {
    int x = 0;
    int y;
    if (x == 1) y = 1; else y = 2;
    x = x + 1;
    int arg;
    if (x < y) arg = x; else arg = 2;
    return f(arg) == 9;
}
