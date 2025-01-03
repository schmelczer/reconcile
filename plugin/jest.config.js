module.exports = {
	testEnvironment: "node",
	moduleFileExtensions: ["js", "ts"],
	testMatch: ["**/src/**/*.test.ts"],
	transform: {
		"^.+\\.(ts|tsx)$": "ts-jest",
	},
};
