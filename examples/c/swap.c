
int main() {
    int a = 1;
    int b = 2;
    {
        int t = a;
        a = b;
        b = t;
    }
    while (b == 1) {
        int t = a;
        a = b;
        b = t;
    }
    return a * 10 + b;
}
