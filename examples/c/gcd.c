int gcd(int a, int b) {
    if (a > 0) a = a; else a = -a;
    if (b > 0) b = b; else b = -b;
    while (a != b) {
        if (a > b) a = a - b;
        else b = b - a;
    }
    return a;
}
int main() {
    return gcd(18, 21) == 3;
}
