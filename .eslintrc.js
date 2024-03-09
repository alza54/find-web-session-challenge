module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2020,
    project: './tsconfig.json',
    extraFileExtensions: ['.json']
  },
  env: {
    es6: true,
    node: true,
    browser: true
  },
  plugins: [
    '@typescript-eslint',
    'json'
  ],
  overrides: [
    {
      files: ['.json'],
      extends: ['plugin:json/recommended']
    },
    {
      files: ['.js', '.jsx', '.ts', '.tsx'],
      extends: 'standard-with-typescript',
    },
    {
      files: ['.ts', '.tsx'],
      rules: {
        '@typescript-eslint/indent': ['error', 2],
        '@typescript-eslint/no-unused-vars': ['warn', { 'argsIgnorePattern': '^_' }],
        '@typescript-eslint/semi': ['error', 'always'],
      },
    }
  ],
  rules: {
    'comma-dangle': ['error', {
      'arrays': 'always-multiline',
      'objects': 'always-multiline',
    }],
    'no-console': 'warn',
    'no-unused-vars': 'off',
    'semi': ['error', 'always'],
  },
};
