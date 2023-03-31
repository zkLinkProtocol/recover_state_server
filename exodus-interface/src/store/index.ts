import { configureStore } from '@reduxjs/toolkit'
import { combineReducers } from 'redux'
import home from './home/reducer'
import { useDispatch } from 'react-redux'

const rootReducer = combineReducers({
  home,
})

export type AppDispatch = typeof store.dispatch
export const useAppDispatch: () => AppDispatch = useDispatch

export type RootState = ReturnType<typeof rootReducer>

export const store = configureStore({
  reducer: rootReducer,
  devTools: process.env.NODE_ENV !== 'production',
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      thunk: true,
      serializableCheck: false,
    }),
})
