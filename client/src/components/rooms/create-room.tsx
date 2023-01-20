import { useZodForm } from "@/lib/hooks/use-zod-form";
import { listingProblemSchema } from "@/types/problem";
import { axios } from "@/utils/axios";
import {
  Button,
  Checkbox,
  FormControl,
  FormErrorMessage,
  FormLabel,
  HStack,
  Input,
  VStack,
} from "@chakra-ui/react";
import { useMutation } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { useCallback, useState } from "react";
import { Controller, FormProvider } from "react-hook-form";
import { z } from "zod";
import { AxiosErrorMessage } from "../ui/axios-error-message";
import { PickProblems } from "./pick-problems";

const SPECIAL_CHARACTERS_REGEX = /[^a-zA-Z0-9_-]/g;

export const createRoomFormSchema = z.object({
  name: z.string().min(3).max(50),
  public: z.boolean(),
  problems: z.array(
    z.union([
      z.object({
        t: z.literal("Category"),
        c: z.object({
          questions: z.number(),
          difficulty: z.number(),
          tags: z.array(z.string()),
        }),
      }),
      z.object({
        t: z.literal("Single"),
        c: z.object({
          id: z.string(),
          data: listingProblemSchema,
        }),
      }),
    ])
  ),
});

export type CreateRoomFormData = z.infer<typeof createRoomFormSchema>;

export const CreateRoom: React.FC = () => {
  const [stage, setStage] = useState<"general" | "problems">("general");
  const form = useZodForm({
    schema: createRoomFormSchema,
    defaultValues: {
      public: true,
      problems: [],
    },
  });
  const errors = form.formState.errors;
  const createRoom = useMutation<unknown, AxiosError, CreateRoomFormData>(
    ["room"],
    (data: CreateRoomFormData) => {
      return axios.post("/room", data);
    }
  );

  const onSubmit = useCallback(
    async (values: CreateRoomFormData) => {
      try {
        await createRoom.mutateAsync(values);
        window.location.href = `/room/${values.name}`;
      } catch {}
    },
    [createRoom]
  );

  return (
    <form onSubmit={form.handleSubmit(onSubmit)}>
      <FormProvider {...form}>
        {stage === "general" && (
          <>
            <VStack gap="2">
              <FormControl isRequired isInvalid={!!errors.name}>
                <FormLabel htmlFor="name">Room Name</FormLabel>
                <Controller
                  name="name"
                  control={form.control}
                  render={({ field: { value, onChange } }) => (
                    <Input
                      id="name"
                      autoComplete="off"
                      value={value || ""}
                      onChange={(e) => {
                        onChange(
                          e.target.value.replace(SPECIAL_CHARACTERS_REGEX, "-")
                        );
                      }}
                    />
                  )}
                />
                <FormErrorMessage>{errors.name?.message}</FormErrorMessage>
              </FormControl>

              <FormControl>
                <Checkbox
                  size="lg"
                  {...form.register("public")}
                  autoComplete="off"
                >
                  Public
                </Checkbox>
              </FormControl>
            </VStack>

            <Button
              my="4"
              w="100%"
              onClick={async () => {
                const fitsSchema = await form.trigger(["name", "public"]);

                if (fitsSchema) setStage("problems");
              }}
            >
              Next
            </Button>
          </>
        )}

        {stage === "problems" && (
          <>
            <PickProblems />

            <HStack>
              <Button my="4" w="100%" onClick={() => setStage("general")}>
                Back
              </Button>
              <Button my="4" w="100%" onClick={form.handleSubmit(onSubmit)}>
                Create Room
              </Button>
            </HStack>
          </>
        )}
        <AxiosErrorMessage error={createRoom.error} />
      </FormProvider>
    </form>
  );
};
