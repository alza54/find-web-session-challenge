module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2020,
    project: './tsconfig.json'
  },
  overrides: [
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
