
int main() {
    int a = 1;
    int b = 0;
    if (a == 0) {
        b = b + 1;
    } else if (a == 1) {
        b = b + 2;
    } else {
        b = b + 3;
    }
    return b == 2;
}
