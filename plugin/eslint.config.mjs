import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import unusedImports from "eslint-plugin-unused-imports";

export default tseslint.config({
	plugins: {
		"unused-imports": unusedImports,
	},
	extends: [eslint.configs.recommended, tseslint.configs.all],
	ignores: ["**/types.ts", "**/*.test.ts"],
	rules: {
		"no-unused-vars": "off",
		"@typescript-eslint/no-unused-vars": "off",
		"@typescript-eslint/no-floating-promises": "error",
		"@typescript-eslint/parameter-properties": "off",
		"@typescript-eslint/require-await": "off",
		"@typescript-eslint/class-methods-use-this": "off",
		"@typescript-eslint/consistent-return": "off",
		"@typescript-eslint/no-unsafe-argument": "off",
		"@typescript-eslint/max-params": [
			"error",
			{
				max: 5,
			},
		],
		"unused-imports/no-unused-imports": "error",
		"@typescript-eslint/no-magic-numbers": "off",
		"@typescript-eslint/prefer-readonly-parameter-types": "off",
		"@typescript-eslint/naming-convention": "off",
		"unused-imports/no-unused-vars": [
			"warn",
			{
				vars: "all",
				varsIgnorePattern: "^_",
				args: "after-used",
				argsIgnorePattern: "^_",
			},
		],
	},
	languageOptions: {
		parserOptions: {
			projectService: true,
			tsconfigRootDir: import.meta.dirname,
		},
	},
});
