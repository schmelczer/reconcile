module.exports = {
	testEnvironment: "node",
	moduleFileExtensions: ["ts"],
	testMatch: ["**/src/**/*.test.ts"],
	transform: {
		"^.+\\.(ts|tsx)$": "ts-jest",
	},
};
