export function badDom(value) {
  document.querySelector('#target').innerHTML = value;
}

export function badStorage(key, value) {
  localStorage.setItem(key, value);
}
