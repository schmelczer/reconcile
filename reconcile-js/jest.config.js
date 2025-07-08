module.exports = {
  preset: 'ts-jest/presets/js-with-babel-esm',
  moduleNameMapper: {
    '^reconcile/reconcile_bg\\.wasm$': `<rootDir>/__mocks__/wasm.js`,
  },
};
