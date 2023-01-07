import { Layout } from "@/components/layout/layout";
import { CreateRoom } from "@/components/rooms/create-room";
import { AxiosErrorMessage } from "@/components/ui/axios-error-message";
import { axios } from "@/utils/axios";
import {
  Box,
  Button,
  Flex,
  Heading,
  HStack,
  Image,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalHeader,
  ModalOverlay,
  Skeleton,
  Text,
  useDisclosure,
  VStack,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { AxiosError } from "axios";
import type { NextPage } from "next";
import { useSession } from "next-auth/react";
import Link from "next/link";
import { FiChevronRight, FiPlus } from "react-icons/fi";

const Home: NextPage = () => {
  const modalDisclosure = useDisclosure();
  const roomsQuery = useQuery<
    {
      name: string;
      owner: {
        id: string;
        name: string;
        image: string;
      };
    }[],
    AxiosError
  >(["room/list"]);
  const { status } = useSession();

  return (
    <Layout title="Radix" selectedPage="rooms">
      <HStack>
        <Heading>Rooms</Heading>
        {status === "authenticated" && (
          <Button
            onClick={modalDisclosure.onOpen}
            ml="auto !important"
            px="8"
            colorScheme="green"
            leftIcon={<FiPlus size={20} />}
          >
            Create
          </Button>
        )}
      </HStack>

      <Modal {...modalDisclosure} size="lg">
        <ModalOverlay />

        <ModalContent>
          <ModalHeader>Create Room</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <CreateRoom />
          </ModalBody>
        </ModalContent>
      </Modal>

      {roomsQuery.status === "loading" && (
        <VStack mt="4">
          {Array(5)
            .fill(0)
            .map((_, i) => (
              <Skeleton key={i} w="100%" h="20" borderRadius="md" />
            ))}
        </VStack>
      )}
      {roomsQuery.status === "error" && (
        <Box mt="4">
          <AxiosErrorMessage error={roomsQuery.error} />
        </Box>
      )}
      {roomsQuery.data?.length === 0 && (
        <Text fontSize="xl" color="whiteAlpha.600" mt="4">
          There are no rooms right now. Create a room!
        </Text>
      )}
      {roomsQuery.status === "success" && (
        <VStack mt="4">
          {roomsQuery.data.map(({ name, owner }) => (
            <Link
              key={name}
              href={`/room/${name}`}
              style={{
                width: "100%",
              }}
            >
              <HStack
                w="100%"
                h="20"
                bg="whiteAlpha.100"
                px="6"
                borderRadius="md"
                gap="2"
                transition="transform 100ms ease-in-out, background 300ms"
                _hover={{
                  cursor: "pointer",
                  transform: "scale(101%)",
                  bg: "whiteAlpha.200",
                }}
              >
                <Heading>{name}</Heading>

                <Text pt="3" fontSize="lg">
                  Host: {owner.name}
                </Text>

                <Box ml="auto !important">
                  <FiChevronRight size={32} />
                </Box>
              </HStack>
            </Link>
          ))}
        </VStack>
      )}
    </Layout>
  );
};

export default Home;
