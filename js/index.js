import init, { tvix_eval } from '../pkg/tvix_eval_wasm.js';

async function run() {
  await init();

  const evalButton = document.getElementById('eval-button');
  const nixExpr = document.getElementById('nix-expr');
  const result = document.getElementById('result');

  evalButton.addEventListener('click', () => {
    const expr = nixExpr.value;
    const res = tvix_eval(expr);
    result.textContent = res;
  });
}

run();