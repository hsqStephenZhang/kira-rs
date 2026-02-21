int g_a[5] = {1, 2, 3, 0, 0};

int main() {
    int a[5] = {1, 2, 3, 4, -5};
    int sum = 0;
    int i = 0;
    while (i < 5) {
        sum = sum + a[i];
        sum = sum + g_a[i];
        i = i + 1;
    }
    return sum;
}
