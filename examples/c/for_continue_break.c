int foo() {
    int sum = 0;
    int i =0;

    while (1) {
        if (i == 5)
            break;
        if (i == 3) {
            i = i +1;
            continue;
        }
        sum = sum + i;
        i = i + 1;
    }

    return sum;
}

int main() {
    return foo() == 7;
}
