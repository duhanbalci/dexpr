function randomBetween(min, max) {
  min = Math.round(min);
  max = Math.round(max);
  return Math.floor(Math.random() * (max - min + 1) + min);
}

function shuffleArray(array) {
  for (let i = array.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [array[i], array[j]] = [array[j], array[i]];
  }
}

function generateRandomArithmeticExpression() {
  const operators = ['+', '-', '*', '/']; // Array of arithmetic operators
  const maxDecimalPlaces = 5;
  const numOperands = randomBetween(10, 20);
  const numParentheses = randomBetween(4, 8);

  // Generate random numbers with two decimal places
  let nums = [];
  for (let i = 0; i < numOperands; i++) {
    nums.push((Math.random() * 100).toFixed(randomBetween(1, maxDecimalPlaces)));
  }

  let res = [];

  shuffleArray(nums);

  let numRemainingOperands = nums.length;
  let openParentheses = 0;

  for (let i = 0; i < nums.length; i++) {
    // Open a parenthesis if there are enough operands remaining
    if (openParentheses < numParentheses && numRemainingOperands > 1 && Math.random() < 0.5) {
      res.push('(');
      openParentheses++;
    }

    res.push(nums[i]);

    // Close a parenthesis if there are enough operands preceding it
    if (openParentheses > 0 && numRemainingOperands > 2 && Math.random() < 0.5) {
      res.push(')');
      openParentheses--;
    }

    if (i < nums.length - 1) {
      res.push(operators[randomBetween(0, 3)]);
    }

    numRemainingOperands--;
  }

  // Close any remaining open parentheses
  while (openParentheses > 0) {
    res.push(')');
    openParentheses--;
  }

  return res.join('');
}

let res = '';

for (let i = 0; i < 100; i++) {
  let expr = generateRandomArithmeticExpression();
  let val = eval(expr);

  // res += `("${expr}", "${val}"),\n`;
  res += `"${expr}",\n`;
}

console.log(res);
