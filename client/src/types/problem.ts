import { PublicUser } from "./user";

export interface Problem {
  id: string;
  title: string;
  author: PublicUser;
  description: string;
  boilerplateCode: BoilerplateCode;
  defaultTestCases: TestCase[];
}

export interface TestCase {
  input: string;
  output: string;
}

export interface BoilerplateCode {
  python: string;
  javascript: string;
}

export interface ListingProblem {
  id: string;
  title: string;
  description: string;
}
