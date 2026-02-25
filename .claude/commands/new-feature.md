## Task

Implement the feature following the following guidelines:

- Analyze the feature and build a developing plan and **Invoke the fullstack-architect subagent** to undersdtand what the plan should be and what are the best practices
- Once the developing plan is define decompose it in a set of atomic steps that the **rust-security-optimizer subagent** can implement
- Ask whatever is not clear and use the answers to clarify the feature. For each answer reanalyze using the **fullstack-architect subagent** and **rust-security-optimizer subagent**.
- Once everything is clear start the development process in this way:
    1. Create a feature branch with this name: "feature/name_of_the_feature" where name_of_the_feature is substituted with the real name of the feature
    2. Start implementing the feature and commit your progress
    3. Review the new code with the **rust-api-reviewer subagent**