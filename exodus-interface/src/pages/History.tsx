import {
  Box,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableFooter,
  TableHead,
  TablePagination,
  TableRow,
  Typography,
  styled,
  useTheme,
} from '@mui/material'
import IconButton from '@mui/material/IconButton'
import FirstPageIcon from '@mui/icons-material/FirstPage'
import KeyboardArrowLeft from '@mui/icons-material/KeyboardArrowLeft'
import KeyboardArrowRight from '@mui/icons-material/KeyboardArrowRight'
import LastPageIcon from '@mui/icons-material/LastPage'
import { Section } from './L2Balances'
import { Header, encryptionAddress } from './Header'
import { useEffect, useState } from 'react'
import { useNetworks, useProofHistory, useTokens } from '../store/home/hooks'
import { useAppDispatch } from '../store'
import { fetchProofHistory } from '../store/home/actions'
import { TokenIcon } from '../components/Icon'

const StyledTableRow = styled(TableRow)({
  transition: 'background .2s ease',
  '&:hover': {
    backgroundColor: 'rgba(0, 0, 0, 0.05)',
  },
})
const TableHeadCell = styled(TableCell)({
  color: 'gray',
  fontSize: 14,
})
const TableBodyCell = styled(TableCell)({
  fontSize: 18,
})

interface TablePaginationActionsProps {
  count: number
  page: number
  rowsPerPage: number
  onPageChange: (event: React.MouseEvent<HTMLButtonElement>, newPage: number) => void
}

function TablePaginationActions(props: TablePaginationActionsProps) {
  const theme = useTheme()
  const { count, page, rowsPerPage, onPageChange } = props

  const handleFirstPageButtonClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    onPageChange(event, 0)
  }

  const handleBackButtonClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    onPageChange(event, page - 1)
  }

  const handleNextButtonClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    onPageChange(event, page + 1)
  }

  const handleLastPageButtonClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    onPageChange(event, Math.max(0, Math.ceil(count / rowsPerPage) - 1))
  }

  return (
    <Box sx={{ flexShrink: 0, ml: 2.5 }}>
      <IconButton
        onClick={handleFirstPageButtonClick}
        disabled={page === 0}
        aria-label="first page"
      >
        {theme.direction === 'rtl' ? <LastPageIcon /> : <FirstPageIcon />}
      </IconButton>
      <IconButton onClick={handleBackButtonClick} disabled={page === 0} aria-label="previous page">
        {theme.direction === 'rtl' ? <KeyboardArrowRight /> : <KeyboardArrowLeft />}
      </IconButton>
      <IconButton
        onClick={handleNextButtonClick}
        disabled={page >= Math.ceil(count / rowsPerPage) - 1}
        aria-label="next page"
      >
        {theme.direction === 'rtl' ? <KeyboardArrowLeft /> : <KeyboardArrowRight />}
      </IconButton>
      <IconButton
        onClick={handleLastPageButtonClick}
        disabled={page >= Math.ceil(count / rowsPerPage) - 1}
        aria-label="last page"
      >
        {theme.direction === 'rtl' ? <FirstPageIcon /> : <LastPageIcon />}
      </IconButton>
    </Box>
  )
}

export const History = () => {
  const [page, setPage] = useState(0)
  const [rowsPerPage, setRowsPerPage] = useState(10)
  const proofHistory = useProofHistory()
  const dispatch = useAppDispatch()
  const tokens = useTokens()
  const networks = useNetworks()

  useEffect(() => {
    dispatch(
      fetchProofHistory({
        page: page,
        proofs_num: rowsPerPage,
      })
    )
  }, [page, rowsPerPage])

  const emptyRows =
    page > 0 ? Math.max(0, (1 + page) * rowsPerPage - (proofHistory?.total_completed_num || 0)) : 0

  const handleChangePage = (event: React.MouseEvent<HTMLButtonElement> | null, newPage: number) => {
    setPage(newPage)
  }

  const handleChangeRowsPerPage = (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    setRowsPerPage(parseInt(event.target.value, 10))
    setPage(0)
  }

  return (
    <>
      <Header />

      <Section>
        <Typography variant="h5">Proof History</Typography>

        <TableContainer>
          <Table sx={{ minWidth: 500 }}>
            <TableHead>
              <StyledTableRow>
                <TableHeadCell sx={{ width: 100 }}>ID</TableHeadCell>
                <TableHeadCell>Token</TableHeadCell>
                <TableHeadCell>Chain</TableHeadCell>
                <TableHeadCell>Address</TableHeadCell>
              </StyledTableRow>
            </TableHead>
            <TableBody>
              {proofHistory?.proofs.map((row) => {
                const { exit_info, proof_info } = row
                const token = tokens[exit_info.l2_source_token]
                return (
                  <StyledTableRow key={proof_info.id}>
                    <TableBodyCell>{proof_info.id}</TableBodyCell>
                    <TableBodyCell>
                      <Stack flexDirection="row" alignItems="center">
                        <TokenIcon symbol={token?.symbol} />
                        <span style={{ marginLeft: 10 }}>{token?.symbol}</span>
                      </Stack>
                    </TableBodyCell>
                    <TableBodyCell>
                      {networks?.find((v) => v.layerTwoChainId === exit_info.chain_id)?.name}
                    </TableBodyCell>
                    <TableBodyCell>{encryptionAddress(exit_info.account_address)}</TableBodyCell>
                  </StyledTableRow>
                )
              })}
            </TableBody>
            <TableFooter>
              <StyledTableRow>
                <TablePagination
                  // rowsPerPageOptions={[10, 20, 50]}
                  colSpan={4}
                  count={proofHistory?.total_completed_num || 0}
                  rowsPerPage={rowsPerPage}
                  page={page}
                  SelectProps={{
                    inputProps: {
                      'aria-label': 'rows per page',
                    },
                    native: true,
                  }}
                  onPageChange={handleChangePage}
                  onRowsPerPageChange={handleChangeRowsPerPage}
                  ActionsComponent={TablePaginationActions}
                />
              </StyledTableRow>
            </TableFooter>
          </Table>
        </TableContainer>
      </Section>
    </>
  )
}
