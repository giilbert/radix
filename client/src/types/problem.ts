import { Enum } from "@/utils/enum";
import { z } from "zod";
import { PublicUser } from "./user";

export interface Problem {
  id: string;
  title: string;
  author: PublicUser;
  description: string;
  boilerplateCode: BoilerplateCode;
  defaultTestCases: TestCase[];
  difficulty: number;
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
  difficulty: number;
  author: PublicUser;
  draft: boolean;
}

export const listingProblemSchema = z.object({
  id: z.string(),
  title: z.string(),
  description: z.string(),
  difficulty: z.number(),
  author: z.object({
    name: z.string(),
    id: z.string(),
  }),
});

export type ProblemFilter = Enum<{
  Category: {
    questions: number;
    difficulty: number | -1;
    tags: string[];
  };
  Single: {
    id: string;
  };
}>;
