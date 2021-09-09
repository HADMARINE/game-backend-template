module.exports = {
  env: {
    browser: true,
    node: true,
  },
  extends: ['plugin:@typescript-eslint/recommended', 'plugin:jest/recommended'],
  settings: {
    'import/resolver': {
      'babel-module': {},
    },
  },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    sourceType: 'module',
    ecmaVersion: 7,
    ecmaFeatures: {
      modules: true,
      jsx: true,
    },
  },
  plugins: ['babel', '@typescript-eslint', 'jest'],
  rules: {
    '@typescript-eslint/array-type': 'error',
    '@typescript-eslint/no-explicit-any': 'off',
    '@typescript-eslint/no-parameter-properties': 'off',
    '@typescript-eslint/no-use-before-define': 'off',
    '@typescript-eslint/prefer-for-of': 'error',
    '@typescript-eslint/prefer-function-type': 'error',
    '@typescript-eslint/unified-signatures': 'error',
    '@typescript-eslint/camelcase': 'off',
    'no-use-before-define': ['error', { variables: false }],
    camelcase: 'off',
    complexity: 'off',
    'constructor-super': 'error',
    'dot-notation': 'error',
    eqeqeq: ['error', 'smart'],
    'guard-for-in': 'error',
    'id-match': 'error',
    'max-classes-per-file': ['error', 1],
    'max-len': ['off'],
    'new-parens': 'error',
    'no-bitwise': 'error',
    'no-caller': 'error',
    'no-cond-assign': 'error',
    'no-console': [
      'warn',
      {
        allow: [
          'warn',
          'dir',
          'timeLog',
          'assert',
          'clear',
          'count',
          'countReset',
          'group',
          'groupEnd',
          'table',
          'dirxml',
          'error',
          'groupCollapsed',
          'Console',
          'profile',
          'profileEnd',
          'timeStamp',
          'context',
        ],
      },
    ],
    'no-debugger': 'error',
    'no-empty': 'off',
    'no-eval': 'error',
    'no-fallthrough': 'off',
    'no-invalid-this': 'off',
    'no-multiple-empty-lines': 'off',
    'no-new-wrappers': 'error',
    'no-shadow': 'off',
    '@typescript-eslint/no-shadow': ['error'],
    'id-blacklist': 'off',
    '@typescript-eslint/type-annotation-spacing': 'warn',
    'no-throw-literal': 'error',
    'no-trailing-spaces': 'error',
    'no-undef-init': 'error',
    'no-underscore-dangle': 'off',
    'no-unsafe-finally': 'error',
    'no-unused-expressions': 'error',
    'no-unused-labels': 'error',
    'object-shorthand': 'error',
    'no-unused-vars': 'off', // Catches this at @typescript-eslint rules
    'one-var': ['error', 'never'],
    '@typescript-eslint/isolatedModules': 'off',
    radix: 'error',
    'spaced-comment': 'error',
    'use-isnan': 'error',
    'valid-typeof': 'off',
    'jest/valid-expect': [
      'error',
      {
        maxArgs: 2,
      },
    ],
  },
};