module.exports = {
	root: true,

	parserOptions: {
		parser: '@typescript-eslint/parser'
	},

	env: {
		browser: true,
	},

	extends: [
		'plugin:vue/essential',
		'@vue/typescript',
		'plugin:@typescript-eslint/recommended',
		'plugin:vue/essential'
	],

	plugins: [
		'vue',
		'@typescript-eslint',
		'vuetify'
	],

	rules: {
		'@typescript-eslint/no-unused-vars': [ 'error', { 'varsIgnorePattern': '^_', 'argsIgnorePattern': '^_' } ],
		'@typescript-eslint/explicit-function-return-type': [ 'error' ],
		'array-bracket-spacing': [ 'error', 'always' ],
		'array-callback-return': 1,
		'arrow-parens': [ 'error', 'always' ],
		'comma-spacing': [ 'error', { 'before': false, 'after': true } ],
		'generator-star-spacing': 'off',
		'indent': [ 'error', 'tab' ],
		'key-spacing': [ 1, { 'beforeColon': false, 'afterColon': true, } ],
		'keyword-spacing': [ 'error', { 'before': true } ],
		'max-len': [ 'error', { 'code': 222, 'ignoreComments': true, 'ignoreTemplateLiterals': true } ],
		'no-await-in-loop': 1,
		'no-console': [ 'error', { allow: [ 'warn', 'error' ] } ],
		// 'no-console': 0,
		'no-constructor-return': 1,
		'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
		'no-extra-parens': 1,
		'no-multi-spaces': [ 'error' ],
		'no-multiple-empty-lines': [ 'error', { 'max': 1, 'maxEOF': 0 } ],
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
		'vue/html-indent': [ 'error', 'tab', { 'attribute': 1, 'closeBracket': 0, 'alignAttributesVertically': true, 'ignores': [] } ],
		'vue/html-quotes': [ 'error', 'single' ],
		'vue/mustache-interpolation-spacing': [ 'error', 'always' ],
		'vue/script-indent': [ 'error', 'tab' ],
		'vuetify/no-deprecated-classes': 'error',
	},
};