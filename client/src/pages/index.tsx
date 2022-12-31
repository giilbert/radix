import { Layout } from "@/components/layout/layout";
import { CreateRoom } from "@/components/rooms/create-room";
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
  Text,
  useDisclosure,
  VStack,
} from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import type { NextPage } from "next";
import { useSession } from "next-auth/react";
import Link from "next/link";
import { FiChevronRight } from "react-icons/fi";

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
    }[]
  >(["room/list"]);
  const { status } = useSession();

  return (
    <Layout title="Radix">
      <HStack>
        <Heading>Rooms</Heading>
        {status === "authenticated" && (
          <Button onClick={modalDisclosure.onOpen}>Create</Button>
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

      {roomsQuery.status === "loading" && <Text>Loading..</Text>}
      {roomsQuery.status === "success" && (
        <Flex mt="4">
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
                bg="whiteAlpha.100"
                py="4"
                px="6"
                borderRadius="md"
                gap="2"
                transition="transform 100ms ease-in-out, background 300ms"
                _hover={{
                  cursor: "pointer",
                  transform: "scale(102%)",
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
        </Flex>
      )}
    </Layout>
  );
};

export default Home;
