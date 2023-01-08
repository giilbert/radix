import { useZodForm } from "@/lib/hooks/use-zod-form";
import { Problem } from "@/types/problem";
import { axios } from "@/utils/axios";
import {
  Box,
  Button,
  Flex,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Heading,
  HStack,
  Input,
  Text,
  Textarea,
} from "@chakra-ui/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { useRouter } from "next/router";
import { useCallback } from "react";
import { FiPlus } from "react-icons/fi";
import { z } from "zod";

const formSchema = z.object({
  title: z.string(),
  description: z.string().max(2000),
  testCases: z.array(
    z.object({
      input: z.string().min(1),
      output: z.string().min(1),
    })
  ),
  boilerplateCode: z.object({
    python: z.string(),
    javascript: z.string(),
  }),
  difficulty: z.number().min(0).max(10),
});

type FormData = z.infer<typeof formSchema>;

export const ProblemEditor: React.FC<{
  problem: Problem;
  setIsEditing: React.Dispatch<React.SetStateAction<boolean>>;
}> = ({ problem, setIsEditing }) => {
  const form = useZodForm({
    schema: formSchema,
    defaultValues: problem,
  });
  const updateProblem = useMutation<void, AxiosError, FormData>(
    ["problem"],
    async (data: FormData) => {
      await axios.put(`/problem/${problem.id}`, data);
    }
  );
  const queryClient = useQueryClient();
  const router = useRouter();
  const onSubmit = useCallback(
    async (values: FormData) => {
      await updateProblem
        .mutateAsync(values)
        .then(async () => {
          await queryClient.invalidateQueries([
            `problem/${router.query.id as string}`,
          ]);

          setIsEditing(false);
        })

        .catch(() => 0);
    },
    [updateProblem, setIsEditing, queryClient, router]
  );

  const errors = form.formState.errors;

  return (
    <>
      <Button
        colorScheme="blue"
        onClick={form.handleSubmit(onSubmit)}
        isLoading={updateProblem.isLoading}
      >
        Save and Exit
      </Button>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Flex gap="2" mt="4" mb="32" flexDirection="column">
          <FormControl isRequired isInvalid={!!errors.title}>
            <FormLabel>Title</FormLabel>
            <Input {...form.register("title")} />
            <FormErrorMessage>{errors.title?.message}</FormErrorMessage>
          </FormControl>

          <FormControl isRequired isInvalid={!!errors.description}>
            <FormLabel>Description</FormLabel>
            <Textarea
              {...form.register("description")}
              resize="none"
              height="48"
            />
            <FormErrorMessage>{errors.description?.message}</FormErrorMessage>
          </FormControl>

          <FormControl isRequired isInvalid={!!errors.difficulty}>
            <FormLabel>Difficulty</FormLabel>
            <Input
              {...form.register("difficulty", { valueAsNumber: true })}
              type="number"
            />
            <FormErrorMessage>{errors.difficulty?.message}</FormErrorMessage>
          </FormControl>

          <hr />

          <Heading>Boilerplate code</Heading>

          <FormControl
            isRequired
            isInvalid={!!errors.boilerplateCode?.javascript?.message}
          >
            <FormLabel>JavaScript</FormLabel>
            <Textarea
              {...form.register("boilerplateCode.javascript")}
              resize="none"
              height="48"
              fontFamily="mono"
            />
            <FormErrorMessage>
              {errors.boilerplateCode?.javascript?.message}
            </FormErrorMessage>
          </FormControl>

          <FormControl
            isRequired
            isInvalid={!!errors.boilerplateCode?.python?.message}
          >
            <FormLabel>Python</FormLabel>
            <Textarea
              {...form.register("boilerplateCode.python")}
              resize="none"
              height="48"
              fontFamily="mono"
            />
            <FormErrorMessage>
              {errors.boilerplateCode?.python?.message}
            </FormErrorMessage>
          </FormControl>

          <hr />

          <Heading>Test Cases</Heading>

          <FormControl isRequired isInvalid={!!errors.testCases?.message}>
            <Flex flexDirection="column" gap="2">
              {form.watch("testCases").map(({ input, output }, i) => (
                <Box key={i}>
                  <HStack gap="2">
                    <Text>Input: </Text>
                    <Input
                      defaultValue={input}
                      fontFamily="mono"
                      onChange={(e) => {
                        const old = form.getValues("testCases");
                        old[i].input = e.target.value;
                        form.setValue("testCases", old);
                      }}
                    />
                    <Text>Output: </Text>
                    <Input
                      defaultValue={output}
                      fontFamily="mono"
                      onChange={(e) => {
                        const old = form.getValues("testCases");
                        old[i].output = e.target.value;
                        form.setValue("testCases", old);
                      }}
                    />
                  </HStack>

                  {(form.formState.errors.testCases || [])[i] && (
                    <Text color="red.400">
                      Error:{" "}
                      {(form.formState.errors.testCases || [])[i]?.input
                        ?.message ||
                        (form.formState.errors.testCases || [])[i]?.output
                          ?.message}
                    </Text>
                  )}
                </Box>
              ))}
              <Button
                w="min"
                leftIcon={<FiPlus />}
                onClick={() => {
                  form.setValue("testCases", [
                    ...form.getValues("testCases"),
                    {
                      input: "",
                      output: "",
                    },
                  ]);
                }}
              >
                Create test case
              </Button>
            </Flex>
            <FormErrorMessage>{errors.testCases?.message}</FormErrorMessage>
          </FormControl>
        </Flex>
      </form>
    </>
  );
};
