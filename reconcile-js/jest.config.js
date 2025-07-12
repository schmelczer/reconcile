module.exports = {
  preset: 'ts-jest/presets/js-with-babel-esm',
  moduleNameMapper: {
    '^reconcile-text/reconcile_text_bg\\.wasm$': `<rootDir>/__mocks__/wasm.js`,
  },
};
