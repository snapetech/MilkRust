export function goodDom(value) {
  document.querySelector('#target').textContent = value;
}

export function goodStorage(key, value) {
  const safeKey = `milkrust:${String(key).slice(0, 64)}`;
  localStorage.setItem(safeKey, value);
}
