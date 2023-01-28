import { ListingProblem } from "@/types/problem";
import {
  Box,
  Heading,
  HStack,
  IconButton,
  Input,
  Popover,
  PopoverBody,
  PopoverContent,
  PopoverHeader,
  PopoverTrigger,
  Skeleton,
  Text,
  useDisclosure,
  useFormControl,
  useOutsideClick,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import {
  ChangeEvent,
  createRef,
  Fragment,
  useCallback,
  useReducer,
  useState,
} from "react";
import { useFieldArray, useFormContext, useFormState } from "react-hook-form";
import { FiEdit, FiTrash, FiX } from "react-icons/fi";
import { debounce } from "throttle-debounce";
import { z, ZodError } from "zod";
import { createRoomFormSchema } from "../rooms/create-room";
import { DifficultyTag } from "./difficulty-tag";

export const SingleProblemSelection: React.FC<{ index: number }> = ({
  index,
}) => {
  const inputRef = createRef<HTMLInputElement>();
  const popover = useDisclosure();
  const form = useFormContext<z.infer<typeof createRoomFormSchema>>();
  const problems = useFieldArray({ control: form.control, name: "problems" });

  const { c, t } = problems.fields[index];
  if (t === "Category") throw new Error("unreachable");
  const selectedProblem = c.data as ListingProblem | null;

  const setSelectedProblem = useCallback(
    (problem: ListingProblem | null) => {
      if (problem) {
        problems.update(index, {
          t: "Single",
          c: {
            id: problem.id,
            data: problem,
          },
        });
      } else {
        problems.update(index, {
          t: "Single",
          c: null as any,
        });
      }
    },
    [index, problems]
  );

  const [query, setQuery] = useState("");
  const problemsQuery = useQuery<ListingProblem[]>([
    `problem/search?query=${query}`,
  ]);

  const error: ZodError =
    Array.isArray(form.formState.errors.problems) &&
    form.formState.errors.problems.at(index);

  useOutsideClick({
    ref: inputRef,
    handler: () => {
      popover.onClose();
    },
  });

  return (
    <>
      {selectedProblem ? (
        <HStack gap="2">
          <IconButton
            aria-label="Change problem"
            icon={<FiEdit />}
            size="sm"
            onClick={() => setSelectedProblem(null)}
          />

          <Box borderRadius="md">
            <HStack alignItems="flex-end" mb="1">
              <Text fontSize="xl" fontWeight="bold">
                {selectedProblem.title}{" "}
              </Text>
              <Text color="gray.400" mb="0.5 !important">
                {selectedProblem.author.name}
              </Text>
            </HStack>

            <DifficultyTag difficulty={selectedProblem.difficulty} />
          </Box>

          <IconButton
            aria-label="Remove problem"
            ml="auto !important"
            icon={<FiTrash />}
            size="sm"
            colorScheme="red"
            onClick={() => problems.remove(index)}
          />
        </HStack>
      ) : (
        <HStack gap="2">
          <Input
            placeholder="Select a problem.."
            ref={inputRef}
            onFocus={() => {
              popover.onOpen();
            }}
            onChange={debounce(500, (e: ChangeEvent<HTMLInputElement>) => {
              setQuery(e.target.value);
            })}
          />

          <IconButton
            aria-label="Remove problem"
            ml="auto !important"
            icon={<FiTrash />}
            size="sm"
            colorScheme="red"
            onClick={() => problems.remove(index)}
          />
        </HStack>
      )}

      {error && (
        <Text color="red.400" mt="1">
          Invalid selection
        </Text>
      )}

      <Popover
        initialFocusRef={inputRef}
        placement="bottom-start"
        matchWidth
        {...popover}
      >
        <PopoverTrigger>
          <Box w="full" />
        </PopoverTrigger>
        <PopoverContent w="full" h="80">
          <PopoverHeader>Search</PopoverHeader>

          <PopoverBody overflowY="auto">
            {problemsQuery.data?.length === 0 && (
              <Text fontSize="xl" color="gray.400">
                No problems found :(
              </Text>
            )}
            {problemsQuery.isLoading && (
              <Text fontSize="xl" color="gray.400">
                Loading..
              </Text>
            )}

            {problemsQuery.data?.map((problem) => (
              <Fragment key={problem.id}>
                <Box
                  _hover={{
                    bgColor: "gray.600",
                    cursor: "pointer",
                  }}
                  px="2"
                  py="2"
                  borderRadius="md"
                  onClick={() => {
                    setQuery("");
                    setSelectedProblem(problem);
                    if (inputRef.current)
                      inputRef.current.value = problem.title;
                  }}
                >
                  <HStack alignItems="flex-end" mb="1">
                    <Text fontSize="xl" fontWeight="bold">
                      {problem.title}{" "}
                    </Text>
                    <Text color="gray.400" mb="0.5 !important">
                      {problem.author.name}
                    </Text>
                  </HStack>

                  <DifficultyTag difficulty={problem.difficulty} />
                </Box>
                <hr />
              </Fragment>
            ))}
          </PopoverBody>
        </PopoverContent>
      </Popover>
    </>
  );
};
