// Test JavaScript file for Canopy extraction

class Calculator {
    constructor() {
        this.value = 0;
    }
    
    add(a, b) {
        return a + b;
    }
    
    multiply(x, y) {
        return x * y;
    }
}

function createCalculator() {
    return new Calculator();
}

const arrowAdd = (a, b) => a + b;

export default Calculator;
export { createCalculator, arrowAdd };

// Modified at Mon Feb  2 17:58:39 EST 2026
