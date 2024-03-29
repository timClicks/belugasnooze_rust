module.exports = {
	env: {
		node: true,
	},
	root: true,
	parser: '@typescript-eslint/parser',
	plugins: [
		'@typescript-eslint',
	],
	extends: [
		'eslint:recommended',
		'plugin:@typescript-eslint/recommended',
	],
	rules: {
		'@typescript-eslint/explicit-function-return-type': [ 'error' ],
		'@typescript-eslint/no-unused-vars': [ 'error', { 'varsIgnorePattern': '^_', 'argsIgnorePattern': '^_' } ],
		'array-bracket-spacing': [ 'error', 'always' ],
		'array-callback-return': 1,
		'arrow-body-style': [ 'error', 'as-needed', { 'requireReturnForObjectLiteral': true } ],
		'arrow-parens': [ 'error', 'always' ],
		'comma-spacing': [ 'error', { 'before': false, 'after': true } ],
		'generator-star-spacing': 'off',
		'indent': [ 'error', 'tab' ],
		'key-spacing': [ 1, { 'beforeColon': false, 'afterColon': true, } ],
		'keyword-spacing': [ 'error', { 'before': true } ],
		'max-len': [ 'error', { 'code': 210, 'ignoreComments': true, 'ignoreTemplateLiterals': true } ],
		'no-await-in-loop': 1,
		'no-constructor-return': 1,
		'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
		'no-extra-parens': 1,
		'no-multi-spaces': [ 'error' ],
		'no-multiple-empty-lines': [ 'error', { 'max': 1, 'maxEOF': 0 } ],
		'no-param-reassign': 1,
		'no-return-assign': 1,
		'no-return-await': 1,
		'no-tabs': 0,
		'no-trailing-spaces': [ 'error', { 'skipBlankLines': true } ],
		'no-unused-vars': 'off',
		'object-curly-spacing': [ 'error', 'always' ],
		'quotes': [ 'error', 'single', { 'allowTemplateLiterals': true } ],
		'require-atomic-updates': 1,
		'semi': [ 'error', 'always' ],
		'space-before-blocks': [ 'error', { 'functions': 'always', 'keywords': 'always', 'classes': 'always' } ],
		'space-before-function-paren': [ 2, 'always' ],
		'space-in-parens': [ 'error', 'never' ],
		// 'no-console': [ 'error' ],
		// https://eslint.org/docs/rules/no-await-in-loop
	}
};