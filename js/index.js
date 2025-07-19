import init, { tvix_eval } from '../pkg/tvix_eval_wasm.js';

async function run() {
  await init();

  const nixExpr = document.getElementById('nix-expr');
  const result = document.getElementById('result');

  const evaluateAndDisplay = () => {
    const expr = nixExpr.value;
    const res = tvix_eval(expr);
    result.textContent = res;
  };

  // Evaluate on input change
  nixExpr.addEventListener('input', evaluateAndDisplay);

  // Initial evaluation on page load
  evaluateAndDisplay();
}

run();
